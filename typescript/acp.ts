import { z } from "zod";
import * as schema from "./schema.js";

export * from "./schema.js";

type AnyMessage = AnyRequest | AnyResponse | AnyNotification;

type AnyRequest = {
  jsonrpc: "2.0";
  id: string | number;
  method: string;
  params?: unknown;
};

type AnyResponse = {
  jsonrpc: "2.0";
  id: string | number;
} & Result<unknown>;

type AnyNotification = {
  jsonrpc: "2.0";
  method: string;
  params?: unknown;
};

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
  data?: unknown;
};

type PendingResponse = {
  resolve: (response: unknown) => void;
  reject: (error: ErrorResponse) => void;
};

type MethodConfig = {
  handler: (params: any) => Promise<any>;
  schema?: z.ZodType<any>;
};

class Connection {
  #pendingResponses: Map<string | number, PendingResponse> = new Map();
  #nextRequestId: number = 0;
  #methods: Map<string, MethodConfig>;
  #peerInput: WritableStream<Uint8Array>;
  #writeQueue: Promise<void> = Promise.resolve();
  #textEncoder: TextEncoder;

  constructor(
    methods: Map<string, MethodConfig>,
    peerInput: WritableStream<Uint8Array>,
    peerOutput: ReadableStream<Uint8Array>,
  ) {
    this.#peerInput = peerInput;
    this.#textEncoder = new TextEncoder();
    this.#methods = methods;
    this.#receive(peerOutput);
  }

  async #receive(output: ReadableStream<Uint8Array>) {
    let content = "";
    const decoder = new TextDecoder();
    const reader = output.getReader();
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        const chunk = value;
        content += decoder.decode(chunk, { stream: true });
        const lines = content.split("\n");
        content = lines.pop() || "";

        for (const line of lines) {
          const trimmedLine = line.trim();

          if (trimmedLine) {
            try {
              const message = JSON.parse(trimmedLine);
              await this.#processMessage(message);
            } catch (error) {
              console.error("Failed to parse message:", error);
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  }

  async #processMessage(message: AnyMessage) {
    if ("method" in message && "id" in message) {
      // It's a request
      let response = await this.#tryCallDelegateMethod(
        message.method,
        message.params,
      );

      await this.#sendMessage({
        jsonrpc: "2.0",
        id: message.id,
        ...response,
      });
    } else if ("method" in message && !("id" in message)) {
      // It's a notification
      await this.#tryCallDelegateMethod(message.method, message.params);
    } else if ("id" in message) {
      // It's a response
      this.#handleResponse(message as AnyResponse);
    }
  }

  async #tryCallDelegateMethod(
    method: string,
    params?: unknown,
  ): Promise<Result<unknown>> {
    const methodConfig = this.#methods.get(method);
    if (!methodConfig) {
      return {
        error: { code: -32601, message: `Method not found - '${method}'` },
      };
    }

    try {
      let validatedParams = params;

      // Validate params if we have a schema for this method
      if (methodConfig.schema) {
        const parseResult = methodConfig.schema.safeParse(params);
        if (!parseResult.success) {
          return {
            error: {
              code: -32602,
              message: "Invalid params",
              data: parseResult.error.format(),
            },
          };
        }
        validatedParams = parseResult.data;
      }

      const result = await methodConfig.handler(validatedParams);
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
        const error = new RequestError(
          response.error.code,
          response.error.message,
          response.error.data,
        );
        pendingResponse.reject(error);
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

  async sendNotification<N>(method: string, params?: N): Promise<void> {
    await this.#sendMessage({ jsonrpc: "2.0", method, params });
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
      .catch((error) => {
        // Continue processing writes on error
        console.error("ACP write error:", error);
      });
    return this.#writeQueue;
  }
}

export interface Agent {
  initialize(
    params: schema.InitializeRequest,
  ): Promise<schema.InitializeResponse>;
  newSession(
    params: schema.NewSessionRequest,
  ): Promise<schema.NewSessionResponse>;
  loadSession(
    params: schema.LoadSessionRequest,
  ): Promise<schema.LoadSessionResponse>;
  authenticate(
    params: schema.AuthenticateRequest,
  ): Promise<schema.AuthenticateResponse>;
  prompt(params: schema.PromptRequest): Promise<schema.PromptResponse>;
  cancelled(params: schema.CancelledNotification): Promise<void>;
}

export interface Client {
  writeTextFile(
    params: schema.WriteTextFileRequest,
  ): Promise<schema.WriteTextFileResponse>;
  readTextFile(
    params: schema.ReadTextFileRequest,
  ): Promise<schema.ReadTextFileResponse>;
  requestPermission(
    params: schema.RequestPermissionRequest,
  ): Promise<schema.RequestPermissionResponse>;
  sessionUpdate(params: schema.SessionNotification): Promise<void>;
}

export class AgentConnection {
  #connection: Connection;

  constructor(
    client: Client,
    input: WritableStream<Uint8Array>,
    output: ReadableStream<Uint8Array>,
  ) {
    // Create method configuration map for client methods
    const methods = new Map<string, MethodConfig>([
      [
        schema.CLIENT_METHODS.fs_write_text_file,
        {
          handler: (params) => client.writeTextFile(params),
          schema: schema.writeTextFileRequestSchema,
        },
      ],
      [
        schema.CLIENT_METHODS.fs_read_text_file,
        {
          handler: (params) => client.readTextFile(params),
          schema: schema.readTextFileRequestSchema,
        },
      ],
      [
        schema.CLIENT_METHODS.session_request_permission,
        {
          handler: (params) => client.requestPermission(params),
          schema: schema.requestPermissionRequestSchema,
        },
      ],
      [
        schema.AGENT_METHODS.session_update,
        {
          handler: (params) => client.sessionUpdate(params),
          schema: schema.sessionNotificationSchema,
        },
      ],
    ]);

    this.#connection = new Connection(methods, input, output);
  }

  async initialize(
    params: schema.InitializeRequest,
  ): Promise<schema.InitializeResponse> {
    return await this.#connection.sendRequest(
      schema.AGENT_METHODS.initialize,
      params,
    );
  }

  async newSession(
    params: schema.NewSessionRequest,
  ): Promise<schema.NewSessionResponse> {
    return await this.#connection.sendRequest(
      schema.AGENT_METHODS.session_new,
      params,
    );
  }

  async loadSession(
    params: schema.LoadSessionRequest,
  ): Promise<schema.LoadSessionResponse> {
    return await this.#connection.sendRequest(
      schema.AGENT_METHODS.session_load,
      params,
    );
  }

  async authenticate(
    params: schema.AuthenticateRequest,
  ): Promise<schema.AuthenticateResponse> {
    return await this.#connection.sendRequest(
      schema.AGENT_METHODS.authenticate,
      params,
    );
  }

  async prompt(params: schema.PromptRequest): Promise<schema.PromptResponse> {
    return await this.#connection.sendRequest(
      schema.AGENT_METHODS.session_prompt,
      params,
    );
  }

  async sendCancelled(params: schema.CancelledNotification): Promise<void> {
    return await this.#connection.sendNotification(
      schema.CLIENT_METHODS.session_cancelled,
      params,
    );
  }
}

export class ClientConnection {
  #connection: Connection;

  constructor(
    agent: Agent,
    input: WritableStream<Uint8Array>,
    output: ReadableStream<Uint8Array>,
  ) {
    // Create method configuration map for agent methods
    const methods = new Map<string, MethodConfig>([
      [
        schema.AGENT_METHODS.initialize,
        {
          handler: (params) => agent.initialize(params),
          schema: schema.initializeRequestSchema,
        },
      ],
      [
        schema.AGENT_METHODS.session_new,
        {
          handler: (params) => agent.newSession(params),
          schema: schema.newSessionRequestSchema,
        },
      ],
      [
        schema.AGENT_METHODS.session_load,
        {
          handler: (params) => agent.loadSession(params),
          schema: schema.loadSessionRequestSchema,
        },
      ],
      [
        schema.AGENT_METHODS.authenticate,
        {
          handler: (params) => agent.authenticate(params),
          schema: schema.authenticateRequestSchema,
        },
      ],
      [
        schema.AGENT_METHODS.session_prompt,
        {
          handler: (params) => agent.prompt(params),
          schema: schema.promptRequestSchema,
        },
      ],
      [
        schema.CLIENT_METHODS.session_cancelled,
        {
          handler: (params) => agent.cancelled(params),
          schema: schema.cancelledNotificationSchema,
        },
      ],
    ]);

    this.#connection = new Connection(methods, input, output);
  }

  async writeTextFile(
    params: schema.WriteTextFileRequest,
  ): Promise<schema.WriteTextFileResponse> {
    return await this.#connection.sendRequest(
      schema.CLIENT_METHODS.fs_write_text_file,
      params,
    );
  }

  async readTextFile(
    params: schema.ReadTextFileRequest,
  ): Promise<schema.ReadTextFileResponse> {
    return await this.#connection.sendRequest(
      schema.CLIENT_METHODS.fs_read_text_file,
      params,
    );
  }

  async requestPermission(
    params: schema.RequestPermissionRequest,
  ): Promise<schema.RequestPermissionResponse> {
    return await this.#connection.sendRequest(
      schema.CLIENT_METHODS.session_request_permission,
      params,
    );
  }

  async sendSessionUpdate(params: schema.SessionNotification): Promise<void> {
    return await this.#connection.sendNotification(
      schema.AGENT_METHODS.session_update,
      params,
    );
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
    if (data !== undefined) {
      this.data = data;
    }
  }

  static parseError(details?: string): RequestError {
    return new RequestError(
      -32700,
      "Parse error",
      details ? { details } : undefined,
    );
  }

  static invalidRequest(details?: string): RequestError {
    return new RequestError(
      -32600,
      "Invalid request",
      details ? { details } : undefined,
    );
  }

  static methodNotFound(details?: string): RequestError {
    return new RequestError(
      -32601,
      "Method not found",
      details ? { details } : undefined,
    );
  }

  static invalidParams(details?: string): RequestError {
    return new RequestError(
      -32602,
      "Invalid params",
      details ? { details } : undefined,
    );
  }

  static internalError(details?: string): RequestError {
    return new RequestError(
      -32603,
      "Internal error",
      details ? { details } : undefined,
    );
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
