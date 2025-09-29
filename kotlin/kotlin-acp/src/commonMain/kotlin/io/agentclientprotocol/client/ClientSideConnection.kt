@file:Suppress("unused")

package io.agentclientprotocol.client

import io.agentclientprotocol.agent.Agent
import io.agentclientprotocol.model.AgentMethods
import io.agentclientprotocol.model.AuthenticateRequest
import io.agentclientprotocol.model.AuthenticateResponse
import io.agentclientprotocol.model.CancelNotification
import io.agentclientprotocol.model.ClientMethods
import io.agentclientprotocol.model.CreateTerminalRequest
import io.agentclientprotocol.model.InitializeRequest
import io.agentclientprotocol.model.InitializeResponse
import io.agentclientprotocol.model.KillTerminalCommandRequest
import io.agentclientprotocol.model.LoadSessionRequest
import io.agentclientprotocol.model.LoadSessionResponse
import io.agentclientprotocol.model.NewSessionRequest
import io.agentclientprotocol.model.NewSessionResponse
import io.agentclientprotocol.model.PromptRequest
import io.agentclientprotocol.model.PromptResponse
import io.agentclientprotocol.model.ReadTextFileRequest
import io.agentclientprotocol.model.ReleaseTerminalRequest
import io.agentclientprotocol.model.RequestPermissionRequest
import io.agentclientprotocol.model.SessionNotification
import io.agentclientprotocol.model.SetSessionModeRequest
import io.agentclientprotocol.model.SetSessionModeResponse
import io.agentclientprotocol.model.TerminalOutputRequest
import io.agentclientprotocol.model.WaitForTerminalExitRequest
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
            val response = client.fsReadTextFile(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setRequestHandler(ClientMethods.FS_WRITE_TEXT_FILE) { request ->
            val params = ACPJson.decodeFromString<WriteTextFileRequest>(
                request.params?.toString() ?: "{}"
            )
            val response = client.fsWriteTextFile(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setRequestHandler(ClientMethods.SESSION_REQUEST_PERMISSION) { request ->
            val params = ACPJson.decodeFromString<RequestPermissionRequest>(
                request.params?.toString() ?: "{}"
            )
            val response = client.sessionRequestPermission(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setNotificationHandler(ClientMethods.SESSION_UPDATE) { notification ->
            val params = ACPJson.decodeFromString<SessionNotification>(
                notification.params?.toString() ?: "{}"
            )
            client.sessionUpdate(params)
        }

        protocol.setRequestHandler(ClientMethods.TERMINAL_CREATE) { request ->
            val params = ACPJson.decodeFromString<CreateTerminalRequest>(
                request.params?.toString() ?: "{}"
            )
            val response = client.terminalCreate(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setRequestHandler(ClientMethods.TERMINAL_OUTPUT) { request ->
            val params = ACPJson.decodeFromString<TerminalOutputRequest>(
                request.params?.toString() ?: "{}"
            )
            val response = client.terminalOutput(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setRequestHandler(ClientMethods.TERMINAL_RELEASE) { request ->
            val params = ACPJson.decodeFromString<ReleaseTerminalRequest>(
                request.params?.toString() ?: "{}"
            )
            val response = client.terminalRelease(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setRequestHandler(ClientMethods.TERMINAL_WAIT_FOR_EXIT) { request ->
            val params = ACPJson.decodeFromString<WaitForTerminalExitRequest>(
                request.params?.toString() ?: "{}"
            )
            val response = client.terminalWaitForExit(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.setRequestHandler(ClientMethods.TERMINAL_KILL) { request ->
            val params = ACPJson.decodeFromString<KillTerminalCommandRequest>(
                request.params?.toString() ?: "{}"
            )
            val response = client.terminalKill(params)
            ACPJson.encodeToJsonElement(response)
        }

        protocol.start()
        logger.info { "Client-side connection established" }
    }

    override suspend fun initialize(request: InitializeRequest): InitializeResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(AgentMethods.INITIALIZE, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun authenticate(request: AuthenticateRequest): AuthenticateResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(AgentMethods.AUTHENTICATE, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun sessionNew(request: NewSessionRequest): NewSessionResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(AgentMethods.SESSION_NEW, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun sessionLoad(request: LoadSessionRequest): LoadSessionResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(AgentMethods.SESSION_LOAD, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun sessionSetMode(request: SetSessionModeRequest): SetSessionModeResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(AgentMethods.SESSION_SET_MODE, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun sessionPrompt(request: PromptRequest): PromptResponse {
        val params = ACPJson.encodeToJsonElement(request)
        val responseJson = protocol.sendRequest(AgentMethods.SESSION_PROMPT, params)
        return ACPJson.decodeFromJsonElement(responseJson)
    }

    override suspend fun sessionCancel(notification: CancelNotification) {
        val params = ACPJson.encodeToJsonElement(notification)
        protocol.sendNotification(AgentMethods.SESSION_CANCEL, params)
    }
}