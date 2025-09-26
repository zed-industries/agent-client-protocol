@file:Suppress("unused")

package io.agentclientprotocol.agent

import io.agentclientprotocol.model.ACPJson
import io.agentclientprotocol.agent.Agent
import io.agentclientprotocol.model.AgentMethods
import io.agentclientprotocol.model.AuthenticateRequest
import io.agentclientprotocol.model.CancelNotification
import io.agentclientprotocol.client.Client
import io.agentclientprotocol.model.ClientMethods
import io.agentclientprotocol.model.InitializeRequest
import io.agentclientprotocol.model.LATEST_PROTOCOL_VERSION
import io.agentclientprotocol.model.LoadSessionRequest
import io.agentclientprotocol.model.NewSessionRequest
import io.agentclientprotocol.model.PromptRequest
import io.agentclientprotocol.model.ReadTextFileRequest
import io.agentclientprotocol.model.ReadTextFileResponse
import io.agentclientprotocol.model.RequestPermissionRequest
import io.agentclientprotocol.model.RequestPermissionResponse
import io.agentclientprotocol.model.SessionNotification
import io.agentclientprotocol.model.WriteTextFileRequest
import io.agentclientprotocol.protocol.Protocol
import io.agentclientprotocol.protocol.ProtocolOptions
import io.agentclientprotocol.transport.Transport
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.encodeToJsonElement

private val logger = KotlinLogging.logger {}

/**
 * An agent-side connection to a client.
 *
 * This class provides the agent's view of an ACP connection, allowing
 * agents to communicate with clients. It implements the {@link Client} interface
 * to provide methods for requesting permissions, accessing the file system,
 * and sending session updates.
 *
 * See protocol docs: [Agent](https://agentclientprotocol.com/protocol/overview#agent)
 */
public class AgentSideConnection(
    private val agent: Agent,
    options: ProtocolOptions = ProtocolOptions()
) : Protocol(options), Client {

    override suspend fun connect(transport: Transport) {
        super.connect(transport)
        
        // Set up request handlers for incoming client requests
        setRequestHandler(AgentMethods.INITIALIZE) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<InitializeRequest>(request.params!!)
            } else {
                InitializeRequest(LATEST_PROTOCOL_VERSION)
            }
            val response = agent.initialize(params)
            ACPJson.encodeToJsonElement(response)
        }

        setRequestHandler(AgentMethods.AUTHENTICATE) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<AuthenticateRequest>(request.params!!)
            } else {
                throw IllegalArgumentException("authenticate requires parameters")
            }
            agent.authenticate(params)
            JsonNull // No response body for authenticate
        }

        setRequestHandler(AgentMethods.SESSION_NEW) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<NewSessionRequest>(request.params!!)
            } else {
                throw IllegalArgumentException("session/new requires parameters")
            }
            val response = agent.newSession(params)
            ACPJson.encodeToJsonElement(response)
        }

        setRequestHandler(AgentMethods.SESSION_LOAD) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<LoadSessionRequest>(request.params!!)
            } else {
                throw IllegalArgumentException("session/load requires parameters")
            }
            agent.loadSession(params)
            JsonNull // No response body for loadSession
        }

        setRequestHandler(AgentMethods.SESSION_PROMPT) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<PromptRequest>(request.params!!)
            } else {
                throw IllegalArgumentException("session/prompt requires parameters")
            }
            val response = agent.prompt(params)
            ACPJson.encodeToJsonElement(response)
        }

        setNotificationHandler(AgentMethods.SESSION_CANCEL) { notification ->
            val params = if (notification.params != null) {
                ACPJson.decodeFromJsonElement<CancelNotification>(notification.params!!)
            } else {
                throw IllegalArgumentException("session/cancel requires parameters")
            }
            agent.cancel(params)
        }

        logger.info { "Agent-side connection established" }
    }

    override suspend fun sessionUpdate(notification: SessionNotification) {
        val params = ACPJson.encodeToJsonElement(notification)
        sendNotification(ClientMethods.SESSION_UPDATE, params)
    }

    override suspend fun requestPermission(request: RequestPermissionRequest): RequestPermissionResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = sendRequest(ClientMethods.SESSION_REQUEST_PERMISSION, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun readTextFile(request: ReadTextFileRequest): ReadTextFileResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = sendRequest(ClientMethods.FS_READ_TEXT_FILE, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun writeTextFile(request: WriteTextFileRequest) {
        val params = ACPJson.encodeToJsonElement(request)
        sendRequest(ClientMethods.FS_WRITE_TEXT_FILE, params)
    }
}