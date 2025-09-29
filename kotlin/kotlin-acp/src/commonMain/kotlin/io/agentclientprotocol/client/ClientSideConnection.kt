@file:Suppress("unused")

package io.agentclientprotocol.client

import io.agentclientprotocol.agent.Agent
import io.agentclientprotocol.model.AgentMethods
import io.agentclientprotocol.model.AuthenticateRequest
import io.agentclientprotocol.model.CancelNotification
import io.agentclientprotocol.model.ClientMethods
import io.agentclientprotocol.model.InitializeRequest
import io.agentclientprotocol.model.InitializeResponse
import io.agentclientprotocol.model.LoadSessionRequest
import io.agentclientprotocol.model.NewSessionRequest
import io.agentclientprotocol.model.NewSessionResponse
import io.agentclientprotocol.model.PromptRequest
import io.agentclientprotocol.model.PromptResponse
import io.agentclientprotocol.model.ReadTextFileRequest
import io.agentclientprotocol.model.RequestPermissionRequest
import io.agentclientprotocol.model.SessionNotification
import io.agentclientprotocol.model.WriteTextFileRequest
import io.agentclientprotocol.protocol.Protocol
import io.agentclientprotocol.protocol.ProtocolOptions
import io.agentclientprotocol.rpc.ACPJson
import io.agentclientprotocol.transport.Transport
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.CoroutineScope
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.encodeToJsonElement

private val logger = KotlinLogging.logger {}

/**
 * A client-side connection to an agent.
 *
 * This class provides the client's view of an ACP connection, allowing
 * clients (such as code editors) to communicate with agents. It implements
 * the {@link Agent} to provide methods for initializing sessions, sending
 * prompts, and managing the agent lifecycle.
 *
 * See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
 */
public class ClientSideConnection(
    parentScope: CoroutineScope,
    private val transport: Transport,
    private val client: Client,
    options: ProtocolOptions = ProtocolOptions()
) : Agent {
    private val protocol = Protocol(parentScope, transport, options)

    public fun start() {
        // Set up request handlers for incoming agent requests
        protocol.setRequestHandler(ClientMethods.FS_READ_TEXT_FILE) { request ->
            val params = ACPJson.decodeFromString<ReadTextFileRequest>(
                request.params?.toString() ?: "{}"
            )
            val response = client.readTextFile(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setRequestHandler(ClientMethods.FS_WRITE_TEXT_FILE) { request ->
            val params = ACPJson.decodeFromString<WriteTextFileRequest>(
                request.params?.toString() ?: "{}"
            )
            client.writeTextFile(params)
            JsonNull // No response body for writeTextFile
        }

        protocol.setRequestHandler(ClientMethods.SESSION_REQUEST_PERMISSION) { request ->
            val params = ACPJson.decodeFromString<RequestPermissionRequest>(
                request.params?.toString() ?: "{}"
            )
            val response = client.requestPermission(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setNotificationHandler(ClientMethods.SESSION_UPDATE) { notification ->
            val params = ACPJson.decodeFromString<SessionNotification>(
                notification.params?.toString() ?: "{}"
            )
            client.sessionUpdate(params)
        }

        protocol.start()
        logger.info { "Client-side connection established" }
    }

    override suspend fun initialize(request: InitializeRequest): InitializeResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(AgentMethods.INITIALIZE, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun authenticate(request: AuthenticateRequest) {
        val params = ACPJson.encodeToJsonElement(request)
        protocol.sendRequest(AgentMethods.AUTHENTICATE, params)
    }

    override suspend fun newSession(request: NewSessionRequest): NewSessionResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(AgentMethods.SESSION_NEW, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun loadSession(request: LoadSessionRequest) {
        val params = ACPJson.encodeToJsonElement(request)
        protocol.sendRequest(AgentMethods.SESSION_LOAD, params)
    }

    override suspend fun prompt(request: PromptRequest): PromptResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(AgentMethods.SESSION_PROMPT, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun cancel(notification: CancelNotification) {
        val params = ACPJson.encodeToJsonElement(notification)
        protocol.sendNotification(AgentMethods.SESSION_CANCEL, params)
    }
}