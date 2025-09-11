import { describe, it, expect, beforeEach } from "vitest";
import { TransformStream } from "node:stream/web";
import {
  Agent,
  ClientSideConnection,
  Client,
  AgentSideConnection,
  InitializeRequest,
  InitializeResponse,
  NewSessionRequest,
  NewSessionResponse,
  LoadSessionRequest,
  LoadSessionResponse,
  AuthenticateRequest,
  AuthenticateResponse,
  PromptRequest,
  PromptResponse,
  WriteTextFileRequest,
  WriteTextFileResponse,
  ReadTextFileRequest,
  ReadTextFileResponse,
  RequestPermissionRequest,
  RequestPermissionResponse,
  CancelNotification,
  SessionNotification,
  PROTOCOL_VERSION,
} from "./acp.js";

describe("Connection", () => {
  let clientToAgent: TransformStream<Uint8Array, Uint8Array>;
  let agentToClient: TransformStream<Uint8Array, Uint8Array>;

  beforeEach(() => {
    clientToAgent = new TransformStream();
    agentToClient = new TransformStream();
  });

  it("handles errors in bidirectional communication", async () => {
    // Create client that throws errors
    class TestClient implements Client {
      async writeTextFile(
        _: WriteTextFileRequest,
      ): Promise<WriteTextFileResponse> {
        throw new Error("Write failed");
      }
      async readTextFile(
        _: ReadTextFileRequest,
      ): Promise<ReadTextFileResponse> {
        throw new Error("Read failed");
      }
      async requestPermission(
        _: RequestPermissionRequest,
      ): Promise<RequestPermissionResponse> {
        throw new Error("Permission denied");
      }
      async sessionUpdate(_: SessionNotification): Promise<void> {
        // no-op
      }
    }

    // Create agent that throws errors
    class TestAgent implements Agent {
      async initialize(_: InitializeRequest): Promise<InitializeResponse> {
        throw new Error("Failed to initialize");
      }
      async newSession(_: NewSessionRequest): Promise<NewSessionResponse> {
        throw new Error("Failed to create session");
      }
      async loadSession(_: LoadSessionRequest): Promise<void> {
        throw new Error("Failed to load session");
      }
      async authenticate(_: AuthenticateRequest): Promise<void> {
        throw new Error("Authentication failed");
      }
      async prompt(_: PromptRequest): Promise<PromptResponse> {
        throw new Error("Prompt failed");
      }
      async cancel(_: CancelNotification): Promise<void> {
        // no-op
      }
    }

    // Set up connections
    const agentConnection = new ClientSideConnection(
      () => new TestClient(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = new AgentSideConnection(
      () => new TestAgent(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Test error handling in client->agent direction
    await expect(
      clientConnection.writeTextFile({
        path: "/test.txt",
        content: "test",
        sessionId: "test-session",
      }),
    ).rejects.toThrow();

    // Test error handling in agent->client direction
    await expect(
      agentConnection.newSession({
        cwd: "/test",
        mcpServers: [],
      }),
    ).rejects.toThrow();
  });

  it("handles concurrent requests", async () => {
    let requestCount = 0;

    // Create client
    class TestClient implements Client {
      async writeTextFile(
        _: WriteTextFileRequest,
      ): Promise<WriteTextFileResponse> {
        requestCount++;
        const currentCount = requestCount;
        await new Promise((resolve) => setTimeout(resolve, 40));
        console.log(`Write request ${currentCount} completed`);
        return null;
      }
      async readTextFile(
        params: ReadTextFileRequest,
      ): Promise<ReadTextFileResponse> {
        return { content: `Content of ${params.path}` };
      }
      async requestPermission(
        _: RequestPermissionRequest,
      ): Promise<RequestPermissionResponse> {
        return {
          outcome: {
            outcome: "selected",
            optionId: "allow",
          },
        };
      }
      async sessionUpdate(_: SessionNotification): Promise<void> {
        // no-op
      }
    }

    // Create agent
    class TestAgent implements Agent {
      async initialize(_: InitializeRequest): Promise<InitializeResponse> {
        return {
          protocolVersion: 1,
          agentCapabilities: { loadSession: false },
          authMethods: [],
        };
      }

      async newSession(_: NewSessionRequest): Promise<NewSessionResponse> {
        return {
          sessionId: "test-session",
        };
      }
      async loadSession(_: LoadSessionRequest): Promise<void> {}
      async authenticate(_: AuthenticateRequest): Promise<void> {
        // no-op
      }
      async prompt(_: PromptRequest): Promise<PromptResponse> {
        return { stopReason: "end_turn" };
      }
      async cancel(_: CancelNotification): Promise<void> {
        // no-op
      }
    }

    new ClientSideConnection(
      () => new TestClient(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = new AgentSideConnection(
      () => new TestAgent(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send multiple concurrent requests
    const promises = [
      clientConnection.writeTextFile({
        path: "/file1.txt",
        content: "content1",
        sessionId: "session1",
      }),
      clientConnection.writeTextFile({
        path: "/file2.txt",
        content: "content2",
        sessionId: "session1",
      }),
      clientConnection.writeTextFile({
        path: "/file3.txt",
        content: "content3",
        sessionId: "session1",
      }),
    ];

    const results = await Promise.all(promises);

    // Verify all requests completed successfully
    expect(results).toHaveLength(3);
    expect(results[0]).toBeNull();
    expect(results[1]).toBeNull();
    expect(results[2]).toBeNull();
    expect(requestCount).toBe(3);
  });

  it("handles message ordering correctly", async () => {
    const messageLog: string[] = [];

    // Create client
    class TestClient implements Client {
      async writeTextFile(
        params: WriteTextFileRequest,
      ): Promise<WriteTextFileResponse> {
        messageLog.push(`writeTextFile called: ${params.path}`);
        return null;
      }
      async readTextFile(
        params: ReadTextFileRequest,
      ): Promise<ReadTextFileResponse> {
        messageLog.push(`readTextFile called: ${params.path}`);
        return { content: "test content" };
      }
      async requestPermission(
        params: RequestPermissionRequest,
      ): Promise<RequestPermissionResponse> {
        messageLog.push(`requestPermission called: ${params.toolCall.title}`);
        return {
          outcome: {
            outcome: "selected",
            optionId: "allow",
          },
        };
      }
      async sessionUpdate(params: SessionNotification): Promise<void> {
        messageLog.push("sessionUpdate called");
      }
    }

    // Create agent
    class TestAgent implements Agent {
      async initialize(_: InitializeRequest): Promise<InitializeResponse> {
        return {
          protocolVersion: 1,
          agentCapabilities: { loadSession: false },
          authMethods: [],
        };
      }
      async newSession(
        request: NewSessionRequest,
      ): Promise<NewSessionResponse> {
        messageLog.push(`newSession called: ${request.cwd}`);
        return {
          sessionId: "test-session",
        };
      }
      async loadSession(params: LoadSessionRequest): Promise<void> {
        messageLog.push(`loadSession called: ${params.sessionId}`);
      }
      async authenticate(params: AuthenticateRequest): Promise<void> {
        messageLog.push(`authenticate called: ${params.methodId}`);
      }
      async prompt(params: PromptRequest): Promise<PromptResponse> {
        messageLog.push(`prompt called: ${params.sessionId}`);
        return { stopReason: "end_turn" };
      }
      async cancel(params: CancelNotification): Promise<void> {
        messageLog.push(`cancelled called: ${params.sessionId}`);
      }
    }

    // Set up connections
    const agentConnection = new ClientSideConnection(
      () => new TestClient(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = new AgentSideConnection(
      () => new TestAgent(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send requests in specific order
    await agentConnection.newSession({
      cwd: "/test",
      mcpServers: [],
    });
    await clientConnection.writeTextFile({
      path: "/test.txt",
      content: "test",
      sessionId: "test-session",
    });
    await clientConnection.readTextFile({
      path: "/test.txt",
      sessionId: "test-session",
    });
    await clientConnection.requestPermission({
      sessionId: "test-session",
      toolCall: {
        title: "Execute command",
        kind: "execute",
        status: "pending",
        toolCallId: "tool-123",
        content: [
          {
            type: "content",
            content: {
              type: "text",
              text: "ls -la",
            },
          },
        ],
      },
      options: [
        {
          kind: "allow_once",
          name: "Allow",
          optionId: "allow",
        },
        {
          kind: "reject_once",
          name: "Reject",
          optionId: "reject",
        },
      ],
    });

    // Verify order
    expect(messageLog).toEqual([
      "newSession called: /test",
      "writeTextFile called: /test.txt",
      "readTextFile called: /test.txt",
      "requestPermission called: Execute command",
    ]);
  });

  it("handles notifications correctly", async () => {
    const notificationLog: string[] = [];

    // Create client
    class TestClient implements Client {
      async writeTextFile(
        _: WriteTextFileRequest,
      ): Promise<WriteTextFileResponse> {
        return null;
      }
      async readTextFile(
        _: ReadTextFileRequest,
      ): Promise<ReadTextFileResponse> {
        return { content: "test" };
      }
      async requestPermission(
        _: RequestPermissionRequest,
      ): Promise<RequestPermissionResponse> {
        return {
          outcome: {
            outcome: "selected",
            optionId: "allow",
          },
        };
      }
      async sessionUpdate(notification: SessionNotification): Promise<void> {
        if (
          notification.update &&
          "sessionUpdate" in notification.update &&
          notification.update.sessionUpdate === "agent_message_chunk"
        ) {
          notificationLog.push(
            `agent message: ${(notification.update.content as any).text}`,
          );
        }
      }
    }

    // Create agent
    class TestAgent implements Agent {
      async initialize(_: InitializeRequest): Promise<InitializeResponse> {
        return {
          protocolVersion: 1,
          agentCapabilities: { loadSession: false },
          authMethods: [],
        };
      }
      async newSession(_: NewSessionRequest): Promise<NewSessionResponse> {
        return {
          sessionId: "test-session",
        };
      }
      async loadSession(_: LoadSessionRequest): Promise<void> {}
      async authenticate(_: AuthenticateRequest): Promise<void> {
        // no-op
      }
      async prompt(_: PromptRequest): Promise<PromptResponse> {
        return { stopReason: "end_turn" };
      }
      async cancel(params: CancelNotification): Promise<void> {
        notificationLog.push(`cancelled: ${params.sessionId}`);
      }
    }

    // Create shared instances
    const testClient = () => new TestClient();
    const testAgent = () => new TestAgent();

    // Set up connections
    const agentConnection = new ClientSideConnection(
      testClient,
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = new AgentSideConnection(
      testAgent,
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send notifications
    await clientConnection.sessionUpdate({
      sessionId: "test-session",
      update: {
        sessionUpdate: "agent_message_chunk",
        content: {
          type: "text",
          text: "Hello from agent",
        },
      },
    });

    await agentConnection.cancel({
      sessionId: "test-session",
    });

    // Wait a bit for async handlers
    await new Promise((resolve) => setTimeout(resolve, 50));

    // Verify notifications were received
    expect(notificationLog).toContain("agent message: Hello from agent");
    expect(notificationLog).toContain("cancelled: test-session");
  });

  it("handles initialize method", async () => {
    // Create client
    class TestClient implements Client {
      async writeTextFile(
        _: WriteTextFileRequest,
      ): Promise<WriteTextFileResponse> {
        return null;
      }
      async readTextFile(
        _: ReadTextFileRequest,
      ): Promise<ReadTextFileResponse> {
        return { content: "test" };
      }
      async requestPermission(
        _: RequestPermissionRequest,
      ): Promise<RequestPermissionResponse> {
        return {
          outcome: {
            outcome: "selected",
            optionId: "allow",
          },
        };
      }
      async sessionUpdate(_: SessionNotification): Promise<void> {
        // no-op
      }
    }

    // Create agent
    class TestAgent implements Agent {
      async initialize(params: InitializeRequest): Promise<InitializeResponse> {
        return {
          protocolVersion: params.protocolVersion,
          agentCapabilities: { loadSession: true },
          authMethods: [
            {
              id: "oauth",
              name: "OAuth",
              description: "Authenticate with OAuth",
            },
          ],
        };
      }
      async newSession(_: NewSessionRequest): Promise<NewSessionResponse> {
        return { sessionId: "test-session" };
      }
      async loadSession(_: LoadSessionRequest): Promise<void> {}
      async authenticate(_: AuthenticateRequest): Promise<void> {
        // no-op
      }
      async prompt(_: PromptRequest): Promise<PromptResponse> {
        return { stopReason: "end_turn" };
      }
      async cancel(_: CancelNotification): Promise<void> {
        // no-op
      }
    }

    const agentConnection = new ClientSideConnection(
      () => new TestClient(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    new AgentSideConnection(
      () => new TestAgent(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Test initialize request
    const response = await agentConnection.initialize({
      protocolVersion: PROTOCOL_VERSION,
      clientCapabilities: {
        fs: {
          readTextFile: false,
          writeTextFile: false,
        },
      },
    });

    expect(response.protocolVersion).toBe(PROTOCOL_VERSION);
    expect(response.agentCapabilities?.loadSession).toBe(true);
    expect(response.authMethods).toHaveLength(1);
    expect(response.authMethods?.[0].id).toBe("oauth");
  });

  it("handles extension methods and notifications", async () => {
    const extensionLog: string[] = [];

    // Create client with extension method support
    class TestClient implements Client {
      async writeTextFile(
        _: WriteTextFileRequest,
      ): Promise<WriteTextFileResponse> {
        return null;
      }
      async readTextFile(
        _: ReadTextFileRequest,
      ): Promise<ReadTextFileResponse> {
        return { content: "test" };
      }
      async requestPermission(
        _: RequestPermissionRequest,
      ): Promise<RequestPermissionResponse> {
        return {
          outcome: {
            outcome: "selected",
            optionId: "allow",
          },
        };
      }
      async sessionUpdate(_: SessionNotification): Promise<void> {
        // no-op
      }
      async extMethod(
        method: string,
        params: Record<string, unknown>,
      ): Promise<Record<string, unknown>> {
        if (method === "example.com/ping") {
          return { response: "pong", params };
        }
        throw new Error(`Unknown method: ${method}`);
      }
      async extNotification(
        method: string,
        params: Record<string, unknown>,
      ): Promise<void> {
        extensionLog.push(`client extNotification: ${method}`);
      }
    }

    // Create agent with extension method support
    class TestAgent implements Agent {
      async initialize(_: InitializeRequest): Promise<InitializeResponse> {
        return {
          protocolVersion: PROTOCOL_VERSION,
          agentCapabilities: { loadSession: false },
        };
      }
      async newSession(_: NewSessionRequest): Promise<NewSessionResponse> {
        return { sessionId: "test-session" };
      }
      async authenticate(_: AuthenticateRequest): Promise<void> {
        // no-op
      }
      async prompt(_: PromptRequest): Promise<PromptResponse> {
        return { stopReason: "end_turn" };
      }
      async cancel(_: CancelNotification): Promise<void> {
        // no-op
      }
      async extMethod(
        method: string,
        params: Record<string, unknown>,
      ): Promise<Record<string, unknown>> {
        if (method === "example.com/echo") {
          return { echo: params };
        }
        throw new Error(`Unknown method: ${method}`);
      }
      async extNotification(
        method: string,
        params: Record<string, unknown>,
      ): Promise<void> {
        extensionLog.push(`agent extNotification: ${method}`);
      }
    }

    const agentConnection = new ClientSideConnection(
      () => new TestClient(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = new AgentSideConnection(
      () => new TestAgent(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Test agent calling client extension method
    const clientResponse = await clientConnection.extMethod(
      "example.com/ping",
      {
        data: "test",
      },
    );
    expect(clientResponse).toEqual({
      response: "pong",
      params: { data: "test" },
    });

    // Test client calling agent extension method
    const agentResponse = await agentConnection.extMethod("example.com/echo", {
      message: "hello",
    });
    expect(agentResponse).toEqual({ echo: { message: "hello" } });

    // Test extension notifications
    await clientConnection.extNotification("example.com/client/notify", {
      info: "client notification",
    });
    await agentConnection.extNotification("example.com/agent/notify", {
      info: "agent notification",
    });

    // Wait a bit for async handlers
    await new Promise((resolve) => setTimeout(resolve, 50));

    // Verify notifications were logged
    expect(extensionLog).toContain(
      "client extNotification: example.com/client/notify",
    );
    expect(extensionLog).toContain(
      "agent extNotification: example.com/agent/notify",
    );
  });

  it("handles optional extension methods correctly", async () => {
    // Create client WITHOUT extension methods
    class TestClientWithoutExtensions implements Client {
      async writeTextFile(
        _: WriteTextFileRequest,
      ): Promise<WriteTextFileResponse> {
        return null;
      }
      async readTextFile(
        _: ReadTextFileRequest,
      ): Promise<ReadTextFileResponse> {
        return { content: "test" };
      }
      async requestPermission(
        _: RequestPermissionRequest,
      ): Promise<RequestPermissionResponse> {
        return {
          outcome: {
            outcome: "selected",
            optionId: "allow",
          },
        };
      }
      async sessionUpdate(_: SessionNotification): Promise<void> {
        // no-op
      }
      // Note: No extMethod or extNotification implemented
    }

    // Create agent WITHOUT extension methods
    class TestAgentWithoutExtensions implements Agent {
      async initialize(_: InitializeRequest): Promise<InitializeResponse> {
        return {
          protocolVersion: PROTOCOL_VERSION,
          agentCapabilities: { loadSession: false },
        };
      }
      async newSession(_: NewSessionRequest): Promise<NewSessionResponse> {
        return { sessionId: "test-session" };
      }
      async authenticate(_: AuthenticateRequest): Promise<void> {
        // no-op
      }
      async prompt(_: PromptRequest): Promise<PromptResponse> {
        return { stopReason: "end_turn" };
      }
      async cancel(_: CancelNotification): Promise<void> {
        // no-op
      }
      // Note: No extMethod or extNotification implemented
    }

    const agentConnection = new ClientSideConnection(
      () => new TestClientWithoutExtensions(),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = new AgentSideConnection(
      () => new TestAgentWithoutExtensions(),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Test that calling extension methods on connections without them throws method not found
    try {
      await clientConnection.extMethod("example.com/ping", { data: "test" });
      expect.fail("Should have thrown method not found error");
    } catch (error: any) {
      expect(error.code).toBe(-32601); // Method not found
      expect(error.data.method).toBe("example.com/ping"); // Should show inner method name
    }

    try {
      await agentConnection.extMethod("example.com/echo", { message: "hello" });
      expect.fail("Should have thrown method not found error");
    } catch (error: any) {
      expect(error.code).toBe(-32601); // Method not found
      expect(error.data.method).toBe("example.com/echo"); // Should show inner method name
    }

    // Notifications should be ignored when not implemented (no error thrown)
    await clientConnection.extNotification("example.com/notify", {
      info: "test",
    });
    await agentConnection.extNotification("example.com/notify", {
      info: "test",
    });
  });
});
