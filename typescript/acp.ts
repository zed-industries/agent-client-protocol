import semver from "semver";
import {
  Agent,
  AGENT_METHODS,
  Client,
  CLIENT_METHODS,
  InitializeParams,
  InitializeResponse,
  LATEST_PROTOCOL_VERSION,
  PushToolCallParams,
  PushToolCallResponse,
  ReadTextFileParams,
  ReadTextFileResponse,
  RequestToolCallConfirmationParams,
  RequestToolCallConfirmationResponse,
  SendUserMessageParams,
  StreamAssistantMessageChunkParams,
  UpdateToolCallParams,
  WriteTextFileParams,
} from "./schema.js";

export * from "./schema.js";

type AnyMessage = AnyRequest | AnyResponse;

type AnyRequest = {
  id: number;
  method: string;
  params?: unknown;
};

type AnyResponse = { jsonrpc: "2.0"; id: number } & Result<unknown>;

type Result<T> =
  | {
      result: T;
    }
  | {
      error: ErrorResponse;
    };

type ErrorResponse = {
  code: number;
  message: string;
  data?: { details?: string };
};

type PendingResponse = {
  resolve: (response: unknown) => void;
  reject: (error: ErrorResponse) => void;
};

class Connection<D> {
  #pendingResponses: Map<number, PendingResponse> = new Map();
  #nextRequestId: number = 0;
  #delegate: D;
  #peerInput: WritableStream<Uint8Array>;
  #writeQueue: Promise<void> = Promise.resolve();
  #textEncoder: TextEncoder;

  constructor(
    delegate: D,
    peerInput: WritableStream<Uint8Array>,
    peerOutput: ReadableStream<Uint8Array>,
  ) {
    this.#peerInput = peerInput;
    this.#textEncoder = new TextEncoder();

    this.#delegate = delegate;
    this.#receive(peerOutput);
  }

  async #receive(output: ReadableStream<Uint8Array>) {
    let content = "";
    const decoder = new TextDecoder();
    for await (const chunk of output) {
      content += decoder.decode(chunk, { stream: true });
      const lines = content.split("\n");
      content = lines.pop() || "";

      for (const line of lines) {
        const trimmedLine = line.trim();

        if (trimmedLine) {
          const message = JSON.parse(trimmedLine);
          this.#processMessage(message);
        }
      }
    }
  }

  async #processMessage(message: AnyMessage) {
    if ("method" in message) {
      let response = await this.#tryCallDelegateMethod(
        message.method,
        message.params,
      );

      await this.#sendMessage({
        jsonrpc: "2.0",
        id: message.id,
        ...response,
      });
    } else {
      this.#handleResponse(message);
    }
  }

  async #tryCallDelegateMethod(
    method: string,
    params?: unknown,
  ): Promise<Result<unknown>> {
    const methodName = method as keyof D;
    if (typeof this.#delegate[methodName] !== "function") {
      return {
        error: { code: -32601, message: `Method not found - '${method}'` },
      };
    }

    try {
      const result = await this.#delegate[methodName](params);
      return { result: result ?? null };
    } catch (error: unknown) {
      if (error instanceof RequestError) {
        return error.toResult();
      }

      let details;

      if (error instanceof Error) {
        details = error.message;
      } else if (
        typeof error === "object" &&
        error != null &&
        "message" in error &&
        typeof error.message === "string"
      ) {
        details = error.message;
      }

      return RequestError.internalError(details).toResult();
    }
  }

  #handleResponse(response: AnyResponse) {
    const pendingResponse = this.#pendingResponses.get(response.id);
    if (pendingResponse) {
      if ("result" in response) {
        pendingResponse.resolve(response.result);
      } else if ("error" in response) {
        pendingResponse.reject(response.error);
      }
      this.#pendingResponses.delete(response.id);
    }
  }

  async sendRequest<Req, Resp>(method: string, params?: Req): Promise<Resp> {
    const id = this.#nextRequestId++;
    const responsePromise = new Promise((resolve, reject) => {
      this.#pendingResponses.set(id, { resolve, reject });
    });
    await this.#sendMessage({ jsonrpc: "2.0", id, method, params });
    return responsePromise as Promise<Resp>;
  }

  async #sendMessage(json: AnyMessage) {
    const content = JSON.stringify(json) + "\n";
    this.#writeQueue = this.#writeQueue
      .then(async () => {
        const writer = this.#peerInput.getWriter();
        try {
          await writer.write(this.#textEncoder.encode(content));
        } finally {
          writer.releaseLock();
        }
      })
      .catch(() => {
        // Continue processing writes on error
      });
    return this.#writeQueue;
  }
}

export class AgentConnection implements Agent {
  #connection: Connection<Client>;

  constructor(
    client: (agent: Agent) => Client,
    input: WritableStream<Uint8Array>,
    output: ReadableStream<Uint8Array>,
  ) {
    this.#connection = new Connection(client(this), input, output);
  }

  async initialize(): Promise<InitializeResponse> {
    const result = await this.#connection.sendRequest<
      InitializeParams,
      InitializeResponse
    >(AGENT_METHODS.INITIALIZE, {
      protocolVersion: LATEST_PROTOCOL_VERSION,
    });

    if (
      !semver.satisfies(result.protocolVersion, `^${LATEST_PROTOCOL_VERSION}`)
    ) {
      throw RequestError.invalidRequest(
        `Incompatible versions: Server ^${result.protocolVersion} / Client: ^${LATEST_PROTOCOL_VERSION}`,
      );
    }

    return result;
  }

  async authenticate(): Promise<void> {
    await this.#connection.sendRequest(AGENT_METHODS.AUTHENTICATE);
  }

  async sendUserMessage(params: SendUserMessageParams): Promise<void> {
    await this.#connection.sendRequest(AGENT_METHODS.SEND_USER_MESSAGE, params);
  }

  async cancelSendMessage(): Promise<void> {
    await this.#connection.sendRequest(AGENT_METHODS.CANCEL_SEND_MESSAGE);
  }
}

export class ClientConnection implements Client {
  #connection: Connection<Agent>;

  constructor(
    agent: (client: Client) => Agent,
    input: WritableStream<Uint8Array>,
    output: ReadableStream<Uint8Array>,
  ) {
    this.#connection = new Connection(agent(this), input, output);
  }

  async streamAssistantMessageChunk(
    params: StreamAssistantMessageChunkParams,
  ): Promise<void> {
    await this.#connection.sendRequest(
      CLIENT_METHODS.STREAM_ASSISTANT_MESSAGE_CHUNK,
      params,
    );
  }

  requestToolCallConfirmation(
    params: RequestToolCallConfirmationParams,
  ): Promise<RequestToolCallConfirmationResponse> {
    return this.#connection.sendRequest(
      CLIENT_METHODS.REQUEST_TOOL_CALL_CONFIRMATION,
      params,
    );
  }

  pushToolCall(params: PushToolCallParams): Promise<PushToolCallResponse> {
    return this.#connection.sendRequest(CLIENT_METHODS.PUSH_TOOL_CALL, params);
  }

  async updateToolCall(params: UpdateToolCallParams): Promise<void> {
    await this.#connection.sendRequest(CLIENT_METHODS.UPDATE_TOOL_CALL, params);
  }

  async writeTextFile(params: WriteTextFileParams): Promise<void> {
    await this.#connection.sendRequest(CLIENT_METHODS.WRITE_TEXT_FILE, params);
  }

  async readTextFile(
    params: ReadTextFileParams,
  ): Promise<ReadTextFileResponse> {
    return this.#connection.sendRequest("readTextFile", params);
  }
}

export class RequestError extends Error {
  data?: { details?: string };

  constructor(
    public code: number,
    message: string,
    details?: string,
  ) {
    super(message);
    this.name = "RequestError";
    if (details) {
      this.data = { details };
    }
  }

  static parseError(details?: string): RequestError {
    return new RequestError(-32700, "Parse error", details);
  }

  static invalidRequest(details?: string): RequestError {
    return new RequestError(-32600, "Invalid request", details);
  }

  static methodNotFound(details?: string): RequestError {
    return new RequestError(-32601, "Method not found", details);
  }

  static invalidParams(details?: string): RequestError {
    return new RequestError(-32602, "Invalid params", details);
  }

  static internalError(details?: string): RequestError {
    return new RequestError(-32603, "Internal error", details);
  }

  toResult<T>(): Result<T> {
    return {
      error: {
        code: this.code,
        message: this.message,
        data: this.data,
      },
    };
  }
}
