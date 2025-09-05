package client

import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import schema.*
import transport.Connection
import transport.RequestError
import transport.requireParams
import util.json
import java.io.InputStream
import java.io.OutputStream

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
class AgentSideConnection(
    toAgent: (AgentSideConnection) -> Agent,
    input: OutputStream,   // write to a client
    output: InputStream    // read from a client
) {
    private val conn: Connection
    init {
        val agent = toAgent(this)
        val handler: suspend (String, JsonElement?) -> JsonElement? = { method, params ->
            when (method) {
                AgentMethods.initialize -> json.encodeToJsonElement(InitializeResponse.serializer(),
                    agent.initialize(requireParams(params, InitializeRequest.serializer())))
                AgentMethods.session_new -> json.encodeToJsonElement(NewSessionResponse.serializer(),
                    agent.newSession(requireParams(params, NewSessionRequest.serializer())))
                AgentMethods.session_load -> {
                    agent.loadSession(requireParams(params, LoadSessionRequest.serializer()))
                    JsonNull
                }
                AgentMethods.authenticate -> {
                    agent.authenticate(requireParams(params, AuthenticateRequest.serializer())); JsonNull
                }
                AgentMethods.session_prompt -> json.encodeToJsonElement(PromptResponse.serializer(),
                    agent.prompt(requireParams(params, PromptRequest.serializer())))
                AgentMethods.session_cancel -> {
                    agent.cancel(requireParams(params, CancelNotification.serializer())); JsonNull
                }
                else -> throw RequestError.methodNotFound(method)
            }
        }
        conn = Connection(handler, input, output)
    }

    suspend fun sessionUpdate(params: SessionNotification) {
        conn.sendNotification(ClientMethods.session_update,
            json.encodeToJsonElement(SessionNotification.serializer(), params))
    }

    suspend fun requestPermission(params: RequestPermissionRequest): RequestPermissionResponse =
        conn.sendRequest(
            ClientMethods.session_request_permission,
            json.encodeToJsonElement(RequestPermissionRequest.serializer(), params),
            RequestPermissionResponse.serializer()
        )

    suspend fun readTextFile(params: ReadTextFileRequest): ReadTextFileResponse =
        conn.sendRequest(
            ClientMethods.fs_read_text_file,
            json.encodeToJsonElement(ReadTextFileRequest.serializer(), params),
            ReadTextFileResponse.serializer()
        )

    suspend fun writeTextFile(params: WriteTextFileRequest): Unit =
        conn.sendRequest(
            ClientMethods.fs_write_text_file,
            json.encodeToJsonElement(WriteTextFileRequest.serializer(), params),
            Unit.serializer()
        )

    suspend fun createTerminal(params: CreateTerminalRequest): TerminalHandle {
        val resp: CreateTerminalResponse = conn.sendRequest(
            ClientMethods.terminal_create,
            json.encodeToJsonElement(CreateTerminalRequest.serializer(), params),
            CreateTerminalResponse.serializer()
        )
        return TerminalHandle(resp.terminalID, params.sessionID, conn)
    }
}