@file:Suppress("unused")

package io.agentclientprotocol.protocol

import io.agentclientprotocol.model.ACPJson
import io.agentclientprotocol.transport.Transport
import io.agentclientprotocol.rpc.JsonRpcError
import io.agentclientprotocol.rpc.JsonRpcErrorCode
import io.agentclientprotocol.rpc.JsonRpcNotification
import io.agentclientprotocol.rpc.JsonRpcRequest
import io.agentclientprotocol.rpc.JsonRpcResponse
import io.agentclientprotocol.rpc.RequestId
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.collections.immutable.persistentMapOf
import kotlin.concurrent.Volatile
import kotlinx.coroutines.*
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.*
import kotlinx.serialization.json.decodeFromJsonElement
import kotlin.coroutines.CoroutineContext
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

private val logger = KotlinLogging.logger {}

/**
 * Exception thrown when a request times out.
 */
public class RequestTimeoutException(message: String) : Exception(message)

/**
 * Exception thrown for JSON-RPC protocol errors.
 */
public class JsonRpcException(
    public val code: Int,
    message: String,
    public val data: JsonElement? = null
) : Exception(message)

/**
 * Configuration options for the protocol.
 */
public open class ProtocolOptions(
    /**
     * Default timeout for requests.
     */
    public val requestTimeout: Duration = 60.seconds
)

/**
 * Base protocol implementation handling JSON-RPC communication over a transport.
 *
 * This class manages request/response correlation, notifications, and error handling.
 */
public abstract class Protocol(
    private val options: ProtocolOptions = ProtocolOptions()
) : CoroutineScope {
    
    private val job = SupervisorJob()
    override val coroutineContext: CoroutineContext = Dispatchers.Default + job

    public var transport: Transport? = null
        private set

    @Volatile
    private var pendingRequests = persistentMapOf<RequestId, CompletableDeferred<JsonElement>>()
    @Volatile 
    private var requestIdCounter = 0L

    /**
     * Request handlers for incoming requests.
     */
    @Volatile
    private var requestHandlers = persistentMapOf<String, suspend (JsonRpcRequest) -> JsonElement?>()

    /**
     * Notification handlers for incoming notifications.
     */
    @Volatile
    private var notificationHandlers = persistentMapOf<String, suspend (JsonRpcNotification) -> Unit>()

    /**
     * Connect to a transport and start processing messages.
     */
    public open suspend fun connect(transport: Transport) {
        this.transport = transport
        
        transport.onClose = {
            // Cancel all pending requests
            synchronized(this) {
                pendingRequests.values.forEach { deferred ->
                    deferred.cancel("Transport closed")
                }
                pendingRequests = persistentMapOf()
            }
        }

        // Start processing incoming messages
        launch {
            transport.messages.collect { message ->
                try {
                    handleIncomingMessage(message)
                } catch (e: Exception) {
                    logger.error(e) { "Error processing incoming message: $message" }
                }
            }
        }

        // Handle transport errors
        launch {
            transport.errors.collect { error ->
                logger.error(error) { "Transport error occurred" }
            }
        }

        transport.start()
    }

    /**
     * Send a request and wait for the response.
     */
    public suspend fun sendRequest(
        method: String,
        params: JsonElement? = null,
        timeout: Duration = options.requestTimeout
    ): JsonElement {
        val transport = checkNotNull(this.transport) { "Transport not connected" }
        
        val requestId = RequestId((synchronized(this) {
            requestIdCounter++
            requestIdCounter
        }).toString())
        val deferred = CompletableDeferred<JsonElement>()
        
        synchronized(this) {
            pendingRequests = pendingRequests.put(requestId, deferred)
        }
        
        try {
            val request = JsonRpcRequest(
                id = requestId,
                method = method,
                params = params
            )
            
            val requestJson = ACPJson.encodeToString(request)
            transport.send(requestJson)
            
            return withTimeout(timeout) {
                deferred.await()
            }
        } catch (e: TimeoutCancellationException) {
            synchronized(this) {
                pendingRequests = pendingRequests.remove(requestId)
            }
            throw RequestTimeoutException("Request timed out after $timeout: $method")
        } catch (e: Exception) {
            synchronized(this) {
                pendingRequests = pendingRequests.remove(requestId)
            }
            throw e
        }
    }

    /**
     * Send a notification (no response expected).
     */
    public suspend fun sendNotification(method: String, params: JsonElement? = null) {
        val transport = checkNotNull(this.transport) { "Transport not connected" }
        
        val notification = JsonRpcNotification(
            method = method,
            params = params
        )
        
        val notificationJson = ACPJson.encodeToString(notification)
        transport.send(notificationJson)
    }

    /**
     * Register a handler for incoming requests.
     */
    public fun setRequestHandler(
        method: String, 
        handler: suspend (JsonRpcRequest) -> JsonElement?
    ) {
        synchronized(this) {
            requestHandlers = requestHandlers.put(method, handler)
        }
    }

    /**
     * Register a handler for incoming notifications.
     */
    public fun setNotificationHandler(
        method: String,
        handler: suspend (JsonRpcNotification) -> Unit
    ) {
        synchronized(this) {
            notificationHandlers = notificationHandlers.put(method, handler)
        }
    }

    /**
     * Close the protocol and cleanup resources.
     */
    public open suspend fun close() {
        transport?.close()
        job.cancel()
    }

    private suspend fun handleIncomingMessage(message: String) {
        try {
            val jsonElement = ACPJson.parseToJsonElement(message)
            val jsonObject = jsonElement.jsonObject

            when {
                jsonObject.containsKey("method") && jsonObject.containsKey("id") -> {
                    // Request
                    val request = ACPJson.decodeFromJsonElement<JsonRpcRequest>(jsonElement)
                    handleRequest(request)
                }
                jsonObject.containsKey("method") -> {
                    // Notification
                    val notification = ACPJson.decodeFromJsonElement<JsonRpcNotification>(jsonElement)
                    handleNotification(notification)
                }
                jsonObject.containsKey("result") || jsonObject.containsKey("error") -> {
                    // Response
                    val response = ACPJson.decodeFromJsonElement<JsonRpcResponse>(jsonElement)
                    handleResponse(response)
                }
                else -> {
                    logger.warn { "Unknown message type: $message" }
                }
            }
        } catch (e: Exception) {
            logger.error(e) { "Failed to parse message: $message" }
        }
    }

    private suspend fun handleRequest(request: JsonRpcRequest) {
        val handler = requestHandlers[request.method]
        if (handler != null) {
            try {
                val result = handler(request)
                sendResponse(request.id, result, null)
            } catch (e: Exception) {
                logger.error(e) { "Error handling request ${request.method}" }
                val error = JsonRpcError(
                    code = JsonRpcErrorCode.INTERNAL_ERROR,
                    message = e.message ?: "Internal error"
                )
                sendResponse(request.id, null, error)
            }
        } else {
            val error = JsonRpcError(
                code = JsonRpcErrorCode.METHOD_NOT_FOUND,
                message = "Method not found: ${request.method}"
            )
            sendResponse(request.id, null, error)
        }
    }

    private suspend fun handleNotification(notification: JsonRpcNotification) {
        val handler = notificationHandlers[notification.method]
        if (handler != null) {
            try {
                handler(notification)
            } catch (e: Exception) {
                logger.error(e) { "Error handling notification ${notification.method}" }
            }
        } else {
            logger.debug { "No handler for notification: ${notification.method}" }
        }
    }

    private fun handleResponse(response: JsonRpcResponse) {
        val deferred = synchronized(this) {
            val currentRequests = pendingRequests
            val deferred = currentRequests[response.id]
            pendingRequests = currentRequests.remove(response.id)
            deferred
        }
        if (deferred != null) {
            if (response.error != null) {
                val exception = JsonRpcException(
                    code = response.error.code,
                    message = response.error.message,
                    data = response.error.data
                )
                deferred.completeExceptionally(exception)
            } else {
                deferred.complete(response.result ?: JsonNull)
            }
        } else {
            logger.warn { "Received response for unknown request ID: ${response.id}" }
        }
    }

    private suspend fun sendResponse(
        requestId: RequestId,
        result: JsonElement?,
        error: JsonRpcError?
    ) {
        val transport = checkNotNull(this.transport) { "Transport not connected" }
        
        val response = JsonRpcResponse(
            id = requestId,
            result = result,
            error = error
        )
        
        val responseJson = ACPJson.encodeToString(response)
        transport.send(responseJson)
    }
}