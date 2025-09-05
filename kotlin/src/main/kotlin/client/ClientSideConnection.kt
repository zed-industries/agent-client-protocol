package client

import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import schema.AgentMethods
import schema.AuthenticateRequest
import schema.CancelNotification
import schema.ClientMethods
import schema.CreateTerminalRequest
import schema.CreateTerminalResponse
import schema.InitializeRequest
import schema.InitializeResponse
import schema.KillTerminalRequest
import schema.LoadSessionRequest
import schema.NewSessionRequest
import schema.NewSessionResponse
import schema.PromptRequest
import schema.PromptResponse
import schema.ReadTextFileRequest
import schema.ReadTextFileResponse
import schema.ReleaseTerminalRequest
import schema.RequestPermissionRequest
import schema.RequestPermissionResponse
import schema.SessionNotification
import schema.TerminalOutputRequest
import schema.TerminalOutputResponse
import schema.WaitForTerminalExitRequest
import schema.WaitForTerminalExitResponse
import schema.WriteTextFileRequest
import transport.Connection
import transport.RequestError
import transport.requireParams
import util.UnitSerializer
import util.json
import java.io.InputStream
import java.io.OutputStream

/**
 * A client-side connection to an agent.
 *
 * This class provides the client's view of an ACP connection, allowing
 * clients (such as code editors) to communicate with agents. It implements
 * the {@link Agent} interface to provide methods for initializing sessions, sending
 * prompts, and managing the agent lifecycle.
 *
 * See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
 */
class ClientSideConnection(
    toClient: (Agent) -> Client,
    input: OutputStream,   // write to an agent
    output: InputStream    // read from the agent
) : Agent {

    private val conn: Connection
    private var clientRef: Client? = null

    init {
        val handler: suspend (String, JsonElement?) -> JsonElement? = { method, params ->
            val client = clientRef ?: throw RequestError.internalError("Client not bound")
            when (method) {
                ClientMethods.fs_write_text_file -> json.encodeToJsonElement(
                    Unit.serializer(),
                    client.writeTextFile(requireParams(params, WriteTextFileRequest.serializer()))
                )

                ClientMethods.fs_read_text_file -> json.encodeToJsonElement(
                    ReadTextFileResponse.serializer(),
                    client.readTextFile(requireParams(params, ReadTextFileRequest.serializer()))
                )

                ClientMethods.session_request_permission -> json.encodeToJsonElement(
                    RequestPermissionResponse.serializer(),
                    client.requestPermission(requireParams(params, RequestPermissionRequest.serializer()))
                )

                ClientMethods.session_update -> {
                    client.sessionUpdate(requireParams(params, SessionNotification.serializer())); JsonNull
                }

                ClientMethods.terminal_create -> json.encodeToJsonElement(
                    CreateTerminalResponse.serializer(),
                    client.createTerminal(requireParams(params, CreateTerminalRequest.serializer()))
                )

                ClientMethods.terminal_output -> json.encodeToJsonElement(
                    TerminalOutputResponse.serializer(),
                    client.terminalOutput(requireParams(params, TerminalOutputRequest.serializer()))
                )

                ClientMethods.terminal_release -> {
                    client.releaseTerminal(requireParams(params, ReleaseTerminalRequest.serializer())); JsonNull
                }

                ClientMethods.terminal_wait_for_exit -> json.encodeToJsonElement(
                    WaitForTerminalExitResponse.serializer(),
                    client.waitForTerminalExit(requireParams(params, WaitForTerminalExitRequest.serializer()))
                )

                ClientMethods.terminal_kill -> {
                    client.killTerminal(requireParams(params, KillTerminalRequest.serializer())); JsonNull
                }

                else -> throw RequestError.methodNotFound(method)
            }
        }
        conn = Connection(handler, input, output)
        clientRef = toClient(this)
    }

    override suspend fun initialize(params: InitializeRequest): InitializeResponse =
        conn.sendRequest(
            AgentMethods.initialize,
            json.encodeToJsonElement(InitializeRequest.serializer(), params),
            InitializeResponse.serializer()
        )

    override suspend fun newSession(params: NewSessionRequest): NewSessionResponse =
        conn.sendRequest(
            AgentMethods.session_new,
            json.encodeToJsonElement(NewSessionRequest.serializer(), params),
            NewSessionResponse.serializer()
        )

    override suspend fun loadSession(params: LoadSessionRequest) {
        conn.sendRequest(
            AgentMethods.session_load,
            json.encodeToJsonElement(LoadSessionRequest.serializer(), params),
            UnitSerializer
        )
    }

    override suspend fun authenticate(params: AuthenticateRequest) {
        conn.sendRequest(
            AgentMethods.authenticate,
            json.encodeToJsonElement(AuthenticateRequest.serializer(), params),
            UnitSerializer
        )
    }

    override suspend fun prompt(params: PromptRequest): PromptResponse =
        conn.sendRequest(
            AgentMethods.session_prompt,
            json.encodeToJsonElement(PromptRequest.serializer(), params),
            PromptResponse.serializer()
        )

    override suspend fun cancel(params: CancelNotification) {
        conn.sendNotification(
            AgentMethods.session_cancel,
            json.encodeToJsonElement(CancelNotification.serializer(), params)
        )
    }
}