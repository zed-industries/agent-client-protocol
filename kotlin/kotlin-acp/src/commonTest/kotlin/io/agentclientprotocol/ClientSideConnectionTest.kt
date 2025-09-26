package io.agentclientprotocol

import io.agentclientprotocol.client.Client
import io.agentclientprotocol.client.ClientSideConnection
import io.agentclientprotocol.mock.MockClient
import io.agentclientprotocol.mock.TestTransport
import io.agentclientprotocol.model.*
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.test.runTest
import kotlin.test.*

class ClientSideConnectionTest {
    private lateinit var mockClient: MockClient
    private lateinit var clientTransport: TestTransport
    private lateinit var agentTransport: TestTransport
    private lateinit var clientConnection: ClientSideConnection

    @BeforeTest
    fun setup() {
        mockClient = MockClient()
        val (transport1, transport2) = TestTransport.createPair()
        clientTransport = transport1
        agentTransport = transport2
        clientConnection = ClientSideConnection(mockClient)
    }

    @AfterTest
    fun teardown() = runTest {
        clientTransport.close()
        agentTransport.close()
    }

    // === Connection Tests ===

    @Test
    fun `test connection establishment`() = runTest {
        // When
        clientConnection.connect(clientTransport)

        // Then
        assertTrue(clientTransport.isConnected)
    }

    // === Agent Method Tests (outgoing requests) ===

    @Ignore
    @Test
    fun `test initialize method sends correct request and handles response`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val request = InitializeRequest(
            protocolVersion = LATEST_PROTOCOL_VERSION,
            clientCapabilities = ClientCapabilities(
                fs = FileSystemCapability(readTextFile = true, writeTextFile = true)
            )
        )

        val expectedResponse = InitializeResponse(
            protocolVersion = LATEST_PROTOCOL_VERSION,
            agentCapabilities = AgentCapabilities()
        )

        // When
        val resultDeferred = async {
            clientConnection.initialize(request)
        }
        
        // Simulate agent responding
        val responseJson = """{"jsonrpc":"2.0","id":1,"result":${ACPJson.encodeToString(InitializeResponse.serializer(), expectedResponse)}}"""
        agentTransport.receiveMessage(responseJson)

        // Then
        val result = resultDeferred.await()
        assertEquals(expectedResponse.protocolVersion, result.protocolVersion)
    }

    @Ignore
    @Test
    fun `test authenticate method sends correct request`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val request = AuthenticateRequest(AuthMethodId("test-auth"))

        // When
        val resultDeferred = async {
            clientConnection.authenticate(request)
        }
        
        // Simulate agent responding with success
        val responseJson = """{"jsonrpc":"2.0","id":1,"result":null}"""
        agentTransport.receiveMessage(responseJson)

        // Then - no exception should be thrown
        resultDeferred.await()
    }

    @Ignore
    @Test
    fun `test newSession method sends correct request and handles response`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val request = NewSessionRequest(
            cwd = "/test/path",
            mcpServers = listOf()
        )

        val expectedResponse = NewSessionResponse(SessionId("test-session"))

        // When
        val resultDeferred = async {
            clientConnection.newSession(request)
        }
        
        // Simulate agent responding
        val responseJson = """{"jsonrpc":"2.0","id":1,"result":${ACPJson.encodeToString(NewSessionResponse.serializer(), expectedResponse)}}"""
        agentTransport.receiveMessage(responseJson)

        // Then
        val result = resultDeferred.await()
        assertEquals(expectedResponse.sessionId, result.sessionId)
    }

    @Ignore
    @Test
    fun `test loadSession method sends correct request`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val request = LoadSessionRequest(
            sessionId = SessionId("existing-session"),
            cwd = "/test/path",
            mcpServers = listOf()
        )

        // When
        val resultDeferred = async {
            clientConnection.loadSession(request)
        }
        
        // Simulate agent responding with success
        val responseJson = """{"jsonrpc":"2.0","id":1,"result":null}"""
        agentTransport.receiveMessage(responseJson)

        // Then - no exception should be thrown
        resultDeferred.await()
    }

    @Ignore
    @Test
    fun `test prompt method sends correct request and handles response`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val request = PromptRequest(
            sessionId = SessionId("test-session"),
            prompt = listOf(ContentBlock.Text("Hello, agent!"))
        )

        val expectedResponse = PromptResponse(StopReason.END_TURN)

        // When
        val resultDeferred = async {
            clientConnection.prompt(request)
        }
        
        // Simulate agent responding
        val responseJson = """{"jsonrpc":"2.0","id":1,"result":${ACPJson.encodeToString(PromptResponse.serializer(), expectedResponse)}}"""
        agentTransport.receiveMessage(responseJson)

        // Then
        val result = resultDeferred.await()
        assertEquals(expectedResponse.stopReason, result.stopReason)
    }

    @Test
    fun `test cancel method sends notification`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val notification = CancelNotification(SessionId("test-session"))

        // When
        clientConnection.cancel(notification)

        // Then - notification should be sent without waiting for response
        // This is a fire-and-forget operation
    }

    // === Client Request Handler Tests (incoming requests) ===

    @Test
    fun `test readTextFile handler calls client and returns response`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val expectedContent = "test file content"
        mockClient.readTextFileResult = ReadTextFileResponse(expectedContent)

        val request = ReadTextFileRequest(
            sessionId = SessionId("test-session"),
            path = "/test/file.txt"
        )

        // When - simulate agent sending readTextFile request
        val requestJson = """{"jsonrpc":"2.0","id":42,"method":"fs/read_text_file","params":${
            ACPJson.encodeToString(
                ReadTextFileRequest.serializer(), request)}}"""
        clientTransport.receiveMessage(requestJson)

        // Give some time for processing
        delay(100)

        // Then
        assertEquals(1, mockClient.readTextFileCalls.size)
        assertEquals(request.path, mockClient.readTextFileCalls[0].path)
        assertEquals(request.sessionId, mockClient.readTextFileCalls[0].sessionId)
    }

    @Test
    fun `test writeTextFile handler calls client`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val request = WriteTextFileRequest(
            sessionId = SessionId("test-session"),
            path = "/test/file.txt",
            content = "new content"
        )

        // When - simulate agent sending writeTextFile request
        val requestJson = """{"jsonrpc":"2.0","id":43,"method":"fs/write_text_file","params":${
            ACPJson.encodeToString(
                WriteTextFileRequest.serializer(), request)}}"""
        clientTransport.receiveMessage(requestJson)

        // Give some time for processing
        delay(100)

        // Then
        assertEquals(1, mockClient.writeTextFileCalls.size)
        assertEquals(request.path, mockClient.writeTextFileCalls[0].path)
        assertEquals(request.content, mockClient.writeTextFileCalls[0].content)
    }

    @Test
    fun `test requestPermission handler calls client and returns response`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val toolCallUpdate = ToolCallUpdate(
            toolCallId = ToolCallId("test-call"),
            title = "Test tool call",
            kind = ToolKind.READ,
            status = ToolCallStatus.PENDING
        )

        val request = RequestPermissionRequest(
            sessionId = SessionId("test-session"),
            toolCall = toolCallUpdate,
            options = listOf(
                PermissionOption(
                    optionId = PermissionOptionId("allow"),
                    name = "Allow",
                    kind = PermissionOptionKind.ALLOW_ONCE
                )
            )
        )

        val expectedOutcome = RequestPermissionOutcome.Selected(PermissionOptionId("allow"))
        mockClient.requestPermissionResult = RequestPermissionResponse(expectedOutcome)

        // When - simulate agent sending requestPermission request
        val requestJson = """{"jsonrpc":"2.0","id":44,"method":"session/request_permission","params":${
            ACPJson.encodeToString(
                RequestPermissionRequest.serializer(), request)}}"""
        clientTransport.receiveMessage(requestJson)

        // Give some time for processing
        delay(100)

        // Then
        assertEquals(1, mockClient.requestPermissionCalls.size)
        assertEquals(request.sessionId, mockClient.requestPermissionCalls[0].sessionId)
        assertEquals(request.toolCall, mockClient.requestPermissionCalls[0].toolCall)
    }

    @Test
    fun `test sessionUpdate handler calls client`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val sessionUpdate = SessionUpdate.AgentMessageChunk(
            content = ContentBlock.Text("Hello from agent!")
        )

        val notification = SessionNotification(
            sessionId = SessionId("test-session"),
            update = sessionUpdate
        )

        // When - simulate agent sending sessionUpdate notification
        val notificationJson = """{"jsonrpc":"2.0","method":"session/update","params":${
            ACPJson.encodeToString(
                SessionNotification.serializer(), notification)}}"""
        clientTransport.receiveMessage(notificationJson)

        // Give some time for processing
        delay(100)

        // Then
        assertEquals(1, mockClient.sessionUpdateCalls.size)
        assertEquals(notification.sessionId, mockClient.sessionUpdateCalls[0].sessionId)
        assertEquals(notification.update, mockClient.sessionUpdateCalls[0].update)
    }

    // === Error Handling Tests ===

    @Ignore
    @Test
    fun `test initialize method handles JSON-RPC error response`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val request = InitializeRequest(LATEST_PROTOCOL_VERSION)

        // When
        val resultDeferred = async {
            assertFailsWith<Exception> {
                clientConnection.initialize(request)
            }
        }
        
        // Simulate agent responding with error
        val errorJson = """{"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"Test error"}}"""
        agentTransport.receiveMessage(errorJson)

        // Then
        resultDeferred.await()
    }

    @Test
    fun `test client method exception propagates to agent`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        // Configure mock to throw exception
        val testClient = object : Client {
            override suspend fun readTextFile(request: ReadTextFileRequest): ReadTextFileResponse {
                throw RuntimeException("File not found")
            }
            override suspend fun writeTextFile(request: WriteTextFileRequest) {}
            override suspend fun requestPermission(request: RequestPermissionRequest): RequestPermissionResponse {
                return RequestPermissionResponse(RequestPermissionOutcome.Cancelled)
            }
            override suspend fun sessionUpdate(notification: SessionNotification) {}
        }

        val connectionWithFailingClient = ClientSideConnection(testClient)
        connectionWithFailingClient.connect(agentTransport)

        val request = ReadTextFileRequest(
            sessionId = SessionId("test-session"),
            path = "/nonexistent/file.txt"
        )

        // When - simulate agent sending readTextFile request
        val requestJson = """{"jsonrpc":"2.0","id":42,"method":"fs/read_text_file","params":${
            ACPJson.encodeToString(
                ReadTextFileRequest.serializer(), request)}}"""
        agentTransport.receiveMessage(requestJson)

        // Give time for processing
        delay(100)

        // Then - error response should be sent back (hard to verify without inspecting transport messages)
        // The test verifies that exceptions don't crash the connection
        assertTrue(agentTransport.isConnected)
    }

    @Test
    fun `test transport disconnection during operation`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val request = InitializeRequest(LATEST_PROTOCOL_VERSION)

        // When
        val resultDeferred = async {
            assertFailsWith<Exception> {
                clientConnection.initialize(request)
            }
        }
        
        // Simulate transport disconnection
        clientTransport.close()

        // Then
        resultDeferred.await()
    }

    // === Integration Tests ===

    @Ignore
    @Test
    fun `test full request-response cycle with real JSON serialization`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val initRequest = InitializeRequest(
            protocolVersion = LATEST_PROTOCOL_VERSION,
            clientCapabilities = ClientCapabilities(
                fs = FileSystemCapability(readTextFile = true, writeTextFile = true)
            )
        )

        val expectedInitResponse = InitializeResponse(
            protocolVersion = LATEST_PROTOCOL_VERSION,
            agentCapabilities = AgentCapabilities()
        )

        // When - Initialize
        val initDeferred = async {
            clientConnection.initialize(initRequest)
        }
        
        val initResponseJson = """{"jsonrpc":"2.0","id":1,"result":${ACPJson.encodeToString(InitializeResponse.serializer(), expectedInitResponse)}}"""
        agentTransport.receiveMessage(initResponseJson)
        
        val initResult = initDeferred.await()
        
        // Then - verify initialization succeeded
        assertEquals(expectedInitResponse.protocolVersion, initResult.protocolVersion)

        // When - Create session
        val sessionRequest = NewSessionRequest(
            cwd = "/test/workspace",
            mcpServers = listOf()
        )

        val expectedSessionResponse = NewSessionResponse(SessionId("new-session-123"))

        val sessionDeferred = async {
            clientConnection.newSession(sessionRequest)
        }
        
        val sessionResponseJson = """{"jsonrpc":"2.0","id":2,"result":${ACPJson.encodeToString(NewSessionResponse.serializer(), expectedSessionResponse)}}"""
        agentTransport.receiveMessage(sessionResponseJson)
        
        val sessionResult = sessionDeferred.await()

        // Then
        assertEquals(expectedSessionResponse.sessionId, sessionResult.sessionId)
    }

    @Ignore
    @Test
    fun `test concurrent requests handling`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        val request1 = InitializeRequest(LATEST_PROTOCOL_VERSION)
        val request2 = NewSessionRequest("/test", listOf())

        val response1 = InitializeResponse(LATEST_PROTOCOL_VERSION)
        val response2 = NewSessionResponse(SessionId("concurrent-session"))

        // When - send concurrent requests
        val deferred1 = async { clientConnection.initialize(request1) }
        val deferred2 = async { clientConnection.newSession(request2) }

        // Respond to both requests (order shouldn't matter due to ID correlation)
        val response1Json = """{"jsonrpc":"2.0","id":1,"result":${ACPJson.encodeToString(InitializeResponse.serializer(), response1)}}"""
        val response2Json = """{"jsonrpc":"2.0","id":2,"result":${ACPJson.encodeToString(NewSessionResponse.serializer(), response2)}}"""
        
        agentTransport.receiveMessage(response1Json)
        agentTransport.receiveMessage(response2Json)

        // Then
        val result1 = deferred1.await()
        val result2 = deferred2.await()

        assertEquals(response1.protocolVersion, result1.protocolVersion)
        assertEquals(response2.sessionId, result2.sessionId)
    }

    @Ignore
    @Test
    fun `test bidirectional communication with file operations`() = runTest {
        // Given
        clientConnection.connect(clientTransport)
        clientTransport.start()
        agentTransport.start()

        mockClient.readTextFileResult = ReadTextFileResponse("Hello from file!")

        // When - agent requests file read
        val fileReadRequest = ReadTextFileRequest(
            sessionId = SessionId("test-session"),
            path = "/test/hello.txt"
        )

        val requestJson = """{"jsonrpc":"2.0","id":100,"method":"fs/read_text_file","params":${
            ACPJson.encodeToString(
                ReadTextFileRequest.serializer(), fileReadRequest)}}"""
        clientTransport.receiveMessage(requestJson)

        // Give time for processing
        delay(100)

        // Then
        assertEquals(1, mockClient.readTextFileCalls.size)
        assertEquals("/test/hello.txt", mockClient.readTextFileCalls[0].path)

        // When - client sends prompt request
        val promptRequest = PromptRequest(
            sessionId = SessionId("test-session"),
            prompt = listOf(ContentBlock.Text("Process the file content"))
        )

        val promptResponse = PromptResponse(StopReason.END_TURN)

        val promptDeferred = async {
            clientConnection.prompt(promptRequest)
        }

        val promptResponseJson = """{"jsonrpc":"2.0","id":3,"result":${ACPJson.encodeToString(PromptResponse.serializer(), promptResponse)}}"""
        agentTransport.receiveMessage(promptResponseJson)

        val result = promptDeferred.await()

        // Then
        assertEquals(StopReason.END_TURN, result.stopReason)
    }
}