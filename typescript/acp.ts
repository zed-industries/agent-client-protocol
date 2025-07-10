import {
  Agent,
  AGENT_METHODS,
  Client,
  CLIENT_METHODS,
  Method,
  Result,
} from "./schema.js";

export * from "./schema.js";

type PendingResponse = {
  resolve: (response: unknown) => void;
  reject: (error: unknown) => void;
};

type AnyMessage = AnyRequest | AnyResponse;

type AnyRequest = {
  id: number;
  method: string;
  params: unknown;
};

type AnyResponse = { id: number } & Result<unknown>;

export class Connection<D, P> {
  #pendingResponses: Map<number, PendingResponse> = new Map();
  #nextRequestId: number = 0;
  #delegate: D;
  #delegateMethods: Record<string, Method>;
  #peerInput: WritableStream<Uint8Array>;
  #writeQueue: Promise<void> = Promise.resolve();
  #textEncoder: TextEncoder;

  constructor(
    delegate: (peer: P) => D,
    delegateMethods: Method[],
    peerMethods: Method[],
    peerInput: WritableStream<Uint8Array>,
    peerOutput: ReadableStream<Uint8Array>,
  ) {
    this.#delegateMethods = delegateMethods.reduce<Record<string, Method>>(
      (acc, method) => {
        acc[method.name] = method;
        return acc;
      },
      {},
    );
    this.#peerInput = peerInput;
    this.#textEncoder = new TextEncoder();

    const peer = this as unknown as Record<
      string,
      (params: unknown) => Promise<unknown>
    >;

    for (const { name, paramPayload, responsePayload } of peerMethods) {
      peer[name] = async (params: unknown) => {
        const result = await this.#sendRequest(
          name,
          paramPayload ? params : null,
        );
        return responsePayload ? result : undefined;
      };
    }

    this.#delegate = delegate(this as unknown as P);
    this.#receive(peerOutput);
  }

  static clientToAgent(
    client: (agent: Agent) => Client,
    input: WritableStream<Uint8Array>,
    output: ReadableStream<Uint8Array>,
  ): Agent {
    return new Connection<Client, Agent>(
      client,
      CLIENT_METHODS,
      AGENT_METHODS,
      input,
      output,
    ) as unknown as Agent;
  }

  static agentToClient(
    agent: (client: Client) => Agent,
    input: WritableStream,
    output: ReadableStream,
  ): Client {
    return new Connection<Agent, Client>(
      agent,
      AGENT_METHODS,
      CLIENT_METHODS,
      input,
      output,
    ) as unknown as Client;
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
        id: message.id,
        ...response,
      });
    } else {
      this.#handleResponse(message);
    }
  }

  async #tryCallDelegateMethod(
    method: string,
    params: unknown,
  ): Promise<Result<unknown>> {
    const methodName = method as keyof D;
    if (
      !this.#delegateMethods[method] ||
      typeof this.#delegate[methodName] !== "function"
    ) {
      return {
        error: { code: -32601, message: `Method not found - '${method}'` },
      };
    }

    try {
      let { paramPayload, responsePayload } = this.#delegateMethods[method];
      const result = await this.#delegate[methodName](
        paramPayload ? params : undefined,
      );
      return { result: responsePayload ? result : null };
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

  async #sendRequest(method: string, params: unknown): Promise<unknown> {
    const id = this.#nextRequestId++;
    const responsePromise = new Promise((resolve, reject) => {
      this.#pendingResponses.set(id, { resolve, reject });
    });
    await this.#sendMessage({ id, method, params });
    return responsePromise;
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

export class RequestError extends Error {
  data?: unknown;

  constructor(
    public code: number,
    message: string,
    data?: unknown,
  ) {
    super(message);
    this.name = "RequestError";
    if (data) {
      this.data = data;
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

  static rateLimitExceeded(): RequestError {
    return new RequestError(429, "Rate limit exceeded");
  }

  static toolCallWaitingForConfirmation(): RequestError {
    return new RequestError(1000, "Tool call waiting for confirmation");
  }

  static toolCallRejected(): RequestError {
    return new RequestError(1001, "Tool call was rejected by the user");
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
