@file:Suppress("unused")

package io.agentclientprotocol.agent

import io.agentclientprotocol.client.Client
import io.agentclientprotocol.model.*
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
    private val parentScope: CoroutineScope,
    private val agent: Agent,
    private val transport: Transport,
    options: ProtocolOptions = ProtocolOptions()
) : Client {
    private val protocol = Protocol(parentScope, transport, options)

    public fun start() {

        // Set up request handlers for incoming client requests
        protocol.setRequestHandler(AgentMethods.INITIALIZE) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<InitializeRequest>(request.params)
            } else {
                InitializeRequest(LATEST_PROTOCOL_VERSION)
            }
            val response = agent.initialize(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setRequestHandler(AgentMethods.AUTHENTICATE) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<AuthenticateRequest>(request.params)
            } else {
                throw IllegalArgumentException("authenticate requires parameters")
            }
            agent.authenticate(params)
            JsonNull // No response body for authenticate
        }

        protocol.setRequestHandler(AgentMethods.SESSION_NEW) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<NewSessionRequest>(request.params)
            } else {
                throw IllegalArgumentException("session/new requires parameters")
            }
            val response = agent.newSession(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setRequestHandler(AgentMethods.SESSION_LOAD) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<LoadSessionRequest>(request.params)
            } else {
                throw IllegalArgumentException("session/load requires parameters")
            }
            agent.loadSession(params)
            JsonNull // No response body for loadSession
        }

        protocol.setRequestHandler(AgentMethods.SESSION_PROMPT) { request ->
            val params = if (request.params != null) {
                ACPJson.decodeFromJsonElement<PromptRequest>(request.params)
            } else {
                throw IllegalArgumentException("session/prompt requires parameters")
            }
            val response = agent.prompt(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setNotificationHandler(AgentMethods.SESSION_CANCEL) { notification ->
            val params = if (notification.params != null) {
                ACPJson.decodeFromJsonElement<CancelNotification>(notification.params)
            } else {
                throw IllegalArgumentException("session/cancel requires parameters")
            }
            agent.cancel(params)
        }

        protocol.start()
        logger.info { "Agent-side connection established" }
    }

    override suspend fun sessionUpdate(notification: SessionNotification) {
        val params = ACPJson.encodeToJsonElement(notification)
        protocol.sendNotification(ClientMethods.SESSION_UPDATE, params)
    }

    override suspend fun requestPermission(request: RequestPermissionRequest): RequestPermissionResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(ClientMethods.SESSION_REQUEST_PERMISSION, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun readTextFile(request: ReadTextFileRequest): ReadTextFileResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(ClientMethods.FS_READ_TEXT_FILE, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun writeTextFile(request: WriteTextFileRequest) {
        val params = ACPJson.encodeToJsonElement(request)
        protocol.sendRequest(ClientMethods.FS_WRITE_TEXT_FILE, params)
    }
}