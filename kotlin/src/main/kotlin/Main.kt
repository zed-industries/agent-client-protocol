import client.Agent
import client.AgentSideConnection
import client.Client
import client.ClientSideConnection
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.*
import schema.*
import transport.RequestError
import util.json
import java.io.PipedInputStream
import java.io.PipedOutputStream

// --- Minimal demo client (prints updates, mocks fs + permission) ---
private class ExampleClient : Client {
    override suspend fun requestPermission(params: RequestPermissionRequest): RequestPermissionResponse {
        println("\n🔐 Permission requested: ${params.toolCall.title}")
        println("\nOptions:")
        params.options.forEachIndexed { i, o -> println("  ${i + 1}. ${o.name} (${o.kind})") }
        // Always pick the first option for demo
        return json.decodeFromJsonElement(
            RequestPermissionResponse.serializer(),
            buildJsonObject {
                putJsonObject("outcome") {
                    put("outcome", "selected")
                    put("optionId", params.options.first().optionID)
                }
            }
        )
    }

    override suspend fun sessionUpdate(params: SessionNotification) {
        val u = params.update
        when (u.sessionUpdate) {
            SessionUpdate.AgentMessageChunk -> {
                // Content is ContentUnion?. Safest: inspect as Json.
                val je = u.content
                    ?.let { util.json.encodeToJsonElement(ContentUnion.serializer(), it) }
                        as? JsonObject
                val t = je?.get("type")?.jsonPrimitive?.contentOrNull
                if (t == "text") {
                    val text = je["text"]?.jsonPrimitive?.contentOrNull ?: ""
                    println(text)
                } else {
                    println("[${t ?: "content"}]")
                }
            }
            SessionUpdate.ToolCall ->
                println("\n🔧 ${u.title} (${u.status})")
            SessionUpdate.ToolCallUpdate ->
                println("\n🔧 Tool call `${u.toolCallID}` updated: ${u.status}")
            SessionUpdate.AgentThoughtChunk,
            SessionUpdate.AvailableCommandsUpdate,
            SessionUpdate.Plan,
            SessionUpdate.UserMessageChunk ->
                println("[${u.sessionUpdate.value}]")
        }
    }


    override suspend fun writeTextFile(params: WriteTextFileRequest): Unit {
        System.err.println("[Client] writeTextFile:\n${json.encodeToString(WriteTextFileRequest.serializer(), params)}")
        // Demo: pretend success; response type is usually empty object
        return json.decodeFromJsonElement(Unit.serializer(), buildJsonObject { })
    }

    override suspend fun readTextFile(params: ReadTextFileRequest): ReadTextFileResponse {
        System.err.println("[Client] readTextFile:\n${json.encodeToString(ReadTextFileRequest.serializer(), params)}")
        return json.decodeFromJsonElement(ReadTextFileResponse.serializer(),
            buildJsonObject { put("content", "Mock file content") })
    }
}

// --- Minimal demo agent (echoes a reply, sends a tiny update) ---
private class ExampleAgent(private val conn: AgentSideConnection) : Agent {
    override suspend fun initialize(params: InitializeRequest): InitializeResponse {
        // Return protocol + minimal capabilities
        val resp = buildJsonObject {
            put("protocolVersion", PROTOCOL_VERSION)
            putJsonObject("agentCapabilities") {
                putJsonObject("prompt") { } // keep minimal; extend as needed
            }
            // advertise optional features only if your schema allows
        }
        return json.decodeFromJsonElement(InitializeResponse.serializer(), resp)
    }

    override suspend fun newSession(params: NewSessionRequest): NewSessionResponse {
        val resp = buildJsonObject { put("sessionId", JsonPrimitive("demo-session")) }
        return json.decodeFromJsonElement(NewSessionResponse.serializer(), resp)
    }

    override suspend fun loadSession(params: LoadSessionRequest) { /* no-op */ }

    override suspend fun authenticate(params: AuthenticateRequest) { /* no-op */ }

    override suspend fun prompt(params: PromptRequest): PromptResponse {
        // Stream one fake chunk back to the client
        runCatching {
            val note = buildJsonObject {
                putJsonObject("update") {
                    put("sessionUpdate", "agent_message_chunk")
                    putJsonObject("content") {
                        put("type", "text")
                        put("text", "Hello from agent 👋")
                    }
                }
                put("sessionId", params.sessionID)
            }
            conn.sessionUpdate(json.decodeFromJsonElement(SessionNotification.serializer(), note))
        }
        val resp = buildJsonObject { put("stopReason", "complete") }
        return json.decodeFromJsonElement(PromptResponse.serializer(), resp)
    }

    override suspend fun cancel(params: CancelNotification) { /* no-op */ }
}

fun main(): Unit = runBlocking {
    // In-memory duplex pipes: client <-> agent
    val agentToClientIn = PipedInputStream()
    val agentToClientOut = PipedOutputStream(agentToClientIn)
    val clientToAgentIn = PipedInputStream()
    val clientToAgentOut = PipedOutputStream(clientToAgentIn)

    // Wire sides
    val agentConn = AgentSideConnection(
        toAgent = { Agent -> ExampleAgent(Agent) },
        input = agentToClientOut,   // write to client
        output = clientToAgentIn    // read from client
    )

    val client = ExampleClient()
    val clientConn = ClientSideConnection(
        toClient = { _ -> client },
        input = clientToAgentOut,   // write to agent
        output = agentToClientIn    // read from agent
    )

    try {
        // initialize
        val initReq = json.decodeFromJsonElement(
            InitializeRequest.serializer(),
            buildJsonObject {
                put("protocolVersion", PROTOCOL_VERSION)
                putJsonObject("clientCapabilities") {
                    putJsonObject("fs") {
                        put("readTextFile", true)
                        put("writeTextFile", true)
                    }
                }
            }
        )
        val init = clientConn.initialize(initReq)
        println("✅ Connected to agent (protocol v${init.protocolVersion})")

        // new session
        val newSessionReq = json.decodeFromJsonElement(
            NewSessionRequest.serializer(),
            buildJsonObject {
                put("cwd", System.getProperty("user.dir"))
                putJsonArray("mcpServers") { }
            }
        )
        val session = clientConn.newSession(newSessionReq)
        println("📝 Created session: ${session.sessionID}")
        println("💬 User: Hello, agent!\n")

        // prompt
        val promptReq = json.decodeFromJsonElement(
            PromptRequest.serializer(),
            buildJsonObject {
                put("sessionId", session.sessionID)
                putJsonArray("prompt") {
                    add(buildJsonObject {
                        put("type", "text")
                        put("text", "Hello, agent!")
                    })
                }
            }
        )
        val result = clientConn.prompt(promptReq)
        println("\n\n✅ Agent completed with: ${result.stopReason}")
    } catch (e: RequestError) {
        System.err.println("[Client] JSON-RPC error: ${e.message} (${e.code})")
    } finally {
        // end
    }
}
