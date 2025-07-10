import { describe, it, expect, beforeEach } from "vitest";
import {
  Agent,
  AuthenticateError,
  CancelSendMessageError,
  Client,
  Connection,
  InitializeError,
  InitializeResponse,
  PushToolCallError,
  PushToolCallParams,
  PushToolCallResponse,
  RequestToolCallConfirmationError,
  RequestToolCallConfirmationParams,
  RequestToolCallConfirmationResponse,
  Result,
  SendUserMessageError,
  SendUserMessageParams,
  StreamAssistantMessageChunkParams,
  UpdateToolCallError,
  UpdateToolCallParams,
  VoidResult,
} from "./acp.js";

describe("Connection", () => {
  let clientToAgent: TransformStream;
  let agentToClient: TransformStream;

  beforeEach(() => {
    clientToAgent = new TransformStream();
    agentToClient = new TransformStream();
  });

  it("handles errors in bidirectional communication", async () => {
    // Create client that throws errors
    class TestClient extends StubClient {
      async pushToolCall(
        _: PushToolCallParams,
      ): Promise<Result<PushToolCallResponse, PushToolCallError>> {
        throw new Error("Tool call failed");
      }
    }

    // Create agent that throws errors
    class TestAgent extends StubAgent {
      async initialize(): Promise<Result<InitializeResponse, InitializeError>> {
        throw new Error("Failed to create thread");
      }
    }

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      (agent) => new TestClient(agent),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Test error handling in client->agent direction
    await expect(
      clientConnection.pushToolCall({
        label: "/missing.ts",
        icon: "fileSearch",
      }),
    ).rejects.toThrow();

    // Test error handling in agent->client direction
    await expect(agentConnection.initialize()).rejects.toThrow();
  });

  it("handles concurrent requests", async () => {
    // Create client with delayed responses
    class TestClient extends StubClient {
      toolCall: number = 0;

      async pushToolCall(
        _: PushToolCallParams,
      ): Promise<Result<PushToolCallResponse, PushToolCallError>> {
        this.toolCall++;
        const id = this.toolCall;
        console.log(id);
        await new Promise((resolve) => setTimeout(resolve, 40));
        console.log(id);
        return { ok: { id } };
      }
    }

    // Create agent with delayed responses
    class TestAgent extends StubAgent {}

    Connection.clientToAgent(
      (a) => new TestClient(a),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send multiple concurrent requests
    const promises = [
      clientConnection.pushToolCall({
        label: "Tool Call 1",
        icon: "fileSearch",
      }),
      clientConnection.pushToolCall({
        label: "Tool Call 2",
        icon: "fileSearch",
      }),
      clientConnection.pushToolCall({
        label: "Tool Call 3",
        icon: "fileSearch",
      }),
    ];

    const results = await Promise.all(promises);

    // Verify all requests completed successfully
    expect(results[0]).toMatchObject({ ok: { id: 1 } });
    expect(results[1]).toMatchObject({ ok: { id: 2 } });
    expect(results[2]).toMatchObject({ ok: { id: 3 } });
  });

  it("handles message ordering correctly", async () => {
    const messageLog: string[] = [];

    class TestClient extends StubClient {
      async pushToolCall(
        _: PushToolCallParams,
      ): Promise<Result<PushToolCallResponse, PushToolCallError>> {
        messageLog.push("pushToolCall called");
        return { ok: { id: 0 } };
      }
      async updateToolCall(
        _: UpdateToolCallParams,
      ): Promise<VoidResult<UpdateToolCallError>> {
        messageLog.push("updateToolCall called");
      }
    }

    class TestAgent extends StubAgent {
      async initialize(): Promise<Result<InitializeResponse, InitializeError>> {
        messageLog.push("initialize called");
        return { ok: { isAuthenticated: true } };
      }
    }

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      (client) => new TestClient(client),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send requests in specific order
    await agentConnection.initialize();
    let result = await clientConnection.pushToolCall({
      icon: "folder",
      label: "Folder",
    });
    if ("error" in result) {
      throw new Error(`Failed to push tool call: ${result.error.message}`);
    }
    const { id } = result.ok;
    await clientConnection.updateToolCall({
      content: {
        type: "markdown",
        markdown: "Markdown",
      },
      status: "finished",
      toolCallId: id,
    });

    // Verify order
    expect(messageLog).toEqual([
      "initialize called",
      "pushToolCall called",
      "updateToolCall called",
    ]);
  });
});

class StubAgent implements Agent {
  constructor(private client: Client) {}
  initialize(): Promise<Result<InitializeResponse, InitializeError>> {
    throw new Error("Method not implemented.");
  }
  authenticate(): Promise<VoidResult<AuthenticateError>> {
    throw new Error("Method not implemented.");
  }
  sendUserMessage(
    _: SendUserMessageParams,
  ): Promise<VoidResult<SendUserMessageError>> {
    throw new Error("Method not implemented.");
  }
  cancelSendMessage(): Promise<VoidResult<CancelSendMessageError>> {
    throw new Error("Method not implemented.");
  }
}

class StubClient implements Client {
  constructor(private agent: Agent) {}
  streamAssistantMessageChunk(
    _: StreamAssistantMessageChunkParams,
  ): Promise<void> {
    throw new Error("Method not implemented.");
  }
  requestToolCallConfirmation(
    _: RequestToolCallConfirmationParams,
  ): Promise<
    Result<
      RequestToolCallConfirmationResponse,
      RequestToolCallConfirmationError
    >
  > {
    throw new Error("Method not implemented.");
  }
  pushToolCall(
    _: PushToolCallParams,
  ): Promise<Result<PushToolCallResponse, PushToolCallError>> {
    throw new Error("Method not implemented.");
  }
  updateToolCall(
    _: UpdateToolCallParams,
  ): Promise<VoidResult<UpdateToolCallError>> {
    throw new Error("Method not implemented.");
  }
}
