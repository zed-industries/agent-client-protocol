@file:Suppress("unused")

package io.agentclientprotocol.rpc

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.*
import kotlin.jvm.JvmInline

/**
 * JSON-RPC version constant.
 */
public const val JSONRPC_VERSION: String = "2.0"

/**
 * Request ID for JSON-RPC messages.
 */
@JvmInline
@Serializable
public value class RequestId(public val value: String) {
    override fun toString(): String = value
}

@Serializable
public sealed interface JsonRpcMessage

/**
 * JSON-RPC request message.
 */
@Serializable
public data class JsonRpcRequest(
    val id: RequestId,
    val method: String,
    val params: JsonElement? = null,
    val jsonrpc: String = JSONRPC_VERSION,
) : JsonRpcMessage

/**
 * JSON-RPC notification message.
 */
@Serializable
public data class JsonRpcNotification(
    val method: String,
    val params: JsonElement? = null,
    val jsonrpc: String = JSONRPC_VERSION,
) : JsonRpcMessage

/**
 * JSON-RPC response message.
 */
@Serializable
public data class JsonRpcResponse(
    val id: RequestId,
    val result: JsonElement? = null,
    val error: JsonRpcError? = null,
    val jsonrpc: String = JSONRPC_VERSION,
) : JsonRpcMessage

/**
 * JSON-RPC error object.
 */
@Serializable
public data class JsonRpcError(
    val code: Int,
    val message: String,
    val data: JsonElement? = null
)

/**
 * Standard JSON-RPC error codes.
 */
public object JsonRpcErrorCode {
    public const val PARSE_ERROR: Int = -32700
    public const val INVALID_REQUEST: Int = -32600
    public const val METHOD_NOT_FOUND: Int = -32601
    public const val INVALID_PARAMS: Int = -32602
    public const val INTERNAL_ERROR: Int = -32603
}

@OptIn(ExperimentalSerializationApi::class)
public val ACPJson: Json by lazy {
    Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
        isLenient = true
        explicitNulls = false
    }
}

/**
 * Helper function to decode JSON-RPC messages based on field presence.
 * JSON-RPC 2.0 spec distinguishes message types by which fields are present:
 * - Response: has "id" and ("result" or "error")
 * - Request: has "id" and "method"
 * - Notification: has "method" but no "id"
 */
public fun decodeJsonRpcMessage(jsonString: String): JsonRpcMessage {
    val element = ACPJson.parseToJsonElement(jsonString)
    require(element is JsonObject) { "Expected JSON object" }

    val hasId = element.containsKey("id")
    val hasMethod = element.containsKey("method")
    val hasResult = element.containsKey("result")
    val hasError = element.containsKey("error")

    return when {
        hasId && (hasResult || hasError) -> ACPJson.decodeFromJsonElement(JsonRpcResponse.serializer(), element)
        hasId && hasMethod -> ACPJson.decodeFromJsonElement(JsonRpcRequest.serializer(), element)
        hasMethod -> ACPJson.decodeFromJsonElement(JsonRpcNotification.serializer(), element)
        else -> error("Unable to determine JsonRpcMessage type from JSON structure")
    }
}