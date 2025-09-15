import { z } from "zod";
import * as schema from "./schema.js";
export * from "./schema.js";
export * from "./stream.js";

import type { Stream } from "./stream.js";
import type {
  AnyMessage,
  AnyResponse,
  Result,
  ErrorResponse,
  PendingResponse,
  RequestHandler,
  NotificationHandler,
} from "./jsonrpc.js";

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
export class AgentSideConnection {
  #connection: Connection;

  /**
   * Creates a new agent-side connection to a client.
   *
   * This establishes the communication channel from the agent's perspective
   * following the ACP specification.
   *
   * @param toAgent - A function that creates an Agent handler to process incoming client requests
   * @param stream - The bidirectional message stream for communication. Typically created using
   *                 {@link ndJsonStream} for stdio-based connections.
   *
   * See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)
   */
  constructor(toAgent: (conn: AgentSideConnection) => Agent, stream: Stream) {
    const agent = toAgent(this);

    const requestHandler = async (
      method: string,
      params: unknown,
    ): Promise<unknown> => {
      switch (method) {
        case schema.AGENT_METHODS.initialize: {
          const validatedParams = schema.initializeRequestSchema.parse(params);
          return agent.initialize(validatedParams as schema.InitializeRequest);
        }
        case schema.AGENT_METHODS.session_new: {
          const validatedParams = schema.newSessionRequestSchema.parse(params);
          return agent.newSession(validatedParams as schema.NewSessionRequest);
        }
        case schema.AGENT_METHODS.session_load: {
          if (!agent.loadSession) {
            throw RequestError.methodNotFound(method);
          }
          const validatedParams = schema.loadSessionRequestSchema.parse(params);
          return agent.loadSession(
            validatedParams as schema.LoadSessionRequest,
          );
        }
        case schema.AGENT_METHODS.session_set_mode: {
          if (!agent.setSessionMode) {
            throw RequestError.methodNotFound(method);
          }
          const validatedParams =
            schema.setSessionModeRequestSchema.parse(params);
          const result = await agent.setSessionMode(
            validatedParams as schema.SetSessionModeRequest,
          );
          return result ?? {};
        }
        case schema.AGENT_METHODS.authenticate: {
          const validatedParams =
            schema.authenticateRequestSchema.parse(params);
          const result = await agent.authenticate(
            validatedParams as schema.AuthenticateRequest,
          );
          return result ?? {};
        }
        case schema.AGENT_METHODS.session_prompt: {
          const validatedParams = schema.promptRequestSchema.parse(params);
          return agent.prompt(validatedParams as schema.PromptRequest);
        }
        default:
          if (method.startsWith("_")) {
            if (!agent.extMethod) {
              throw RequestError.methodNotFound(method);
            }
            return agent.extMethod(
              method.substring(1),
              params as Record<string, unknown>,
            );
          }
          throw RequestError.methodNotFound(method);
      }
    };

    const notificationHandler = async (
      method: string,
      params: unknown,
    ): Promise<void> => {
      switch (method) {
        case schema.AGENT_METHODS.session_cancel: {
          const validatedParams = schema.cancelNotificationSchema.parse(params);
          return agent.cancel(validatedParams as schema.CancelNotification);
        }
        default:
          if (method.startsWith("_")) {
            if (!agent.extNotification) {
              return;
            }
            return agent.extNotification(
              method.substring(1),
              params as Record<string, unknown>,
            );
          }
          throw RequestError.methodNotFound(method);
      }
    };

    this.#connection = new Connection(
      requestHandler,
      notificationHandler,
      stream,
    );
  }

  /**
   * Handles session update notifications from the agent.
   *
   * This is a notification endpoint (no response expected) that sends
   * real-time updates about session progress, including message chunks,
   * tool calls, and execution plans.
   *
   * Note: Clients SHOULD continue accepting tool call updates even after
   * sending a `session/cancel` notification, as the agent may send final
   * updates before responding with the cancelled stop reason.
   *
   * See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
   */
  async sessionUpdate(params: schema.SessionNotification): Promise<void> {
    return await this.#connection.sendNotification(
      schema.CLIENT_METHODS.session_update,
      params,
    );
  }

  /**
   * Requests permission from the user for a tool call operation.
   *
   * Called by the agent when it needs user authorization before executing
   * a potentially sensitive operation. The client should present the options
   * to the user and return their decision.
   *
   * If the client cancels the prompt turn via `session/cancel`, it MUST
   * respond to this request with `RequestPermissionOutcome::Cancelled`.
   *
   * See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
   */
  async requestPermission(
    params: schema.RequestPermissionRequest,
  ): Promise<schema.RequestPermissionResponse> {
    return await this.#connection.sendRequest(
      schema.CLIENT_METHODS.session_request_permission,
      params,
    );
  }

  /**
   * Reads content from a text file in the client's file system.
   *
   * Only available if the client advertises the `fs.readTextFile` capability.
   * Allows the agent to access file contents within the client's environment.
   *
   * See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
   */
  async readTextFile(
    params: schema.ReadTextFileRequest,
  ): Promise<schema.ReadTextFileResponse> {
    return await this.#connection.sendRequest(
      schema.CLIENT_METHODS.fs_read_text_file,
      params,
    );
  }

  /**
   * Writes content to a text file in the client's file system.
   *
   * Only available if the client advertises the `fs.writeTextFile` capability.
   * Allows the agent to create or modify files within the client's environment.
   *
   * See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
   */
  async writeTextFile(
    params: schema.WriteTextFileRequest,
  ): Promise<schema.WriteTextFileResponse> {
    return (
      (await this.#connection.sendRequest(
        schema.CLIENT_METHODS.fs_write_text_file,
        params,
      )) ?? {}
    );
  }

  /**
   * Executes a command in a new terminal.
   *
   * Returns a `TerminalHandle` that can be used to get output, wait for exit,
   * kill the command, or release the terminal.
   *
   * The terminal can also be embedded in tool calls by using its ID in
   * `ToolCallContent` with type "terminal".
   *
   * @param params - The terminal creation parameters
   * @returns A handle to control and monitor the terminal
   */
  async createTerminal(
    params: schema.CreateTerminalRequest,
  ): Promise<TerminalHandle> {
    const response = (await this.#connection.sendRequest(
      schema.CLIENT_METHODS.terminal_create,
      params,
    )) as schema.CreateTerminalResponse;

    return new TerminalHandle(
      response.terminalId,
      params.sessionId,
      this.#connection,
    );
  }

  /**
   * Extension method
   *
   * Allows the Agent to send an arbitrary request that is not part of the ACP spec.
   */
  async extMethod(
    method: string,
    params: Record<string, unknown>,
  ): Promise<Record<string, unknown>> {
    return await this.#connection.sendRequest(`_${method}`, params);
  }

  /**
   * Extension notification
   *
   * Allows the Agent to send an arbitrary notification that is not part of the ACP spec.
   */
  async extNotification(
    method: string,
    params: Record<string, unknown>,
  ): Promise<void> {
    return await this.#connection.sendNotification(`_${method}`, params);
  }
}

/**
 * Handle for controlling and monitoring a terminal created via `createTerminal`.
 *
 * Provides methods to:
 * - Get current output without waiting
 * - Wait for command completion
 * - Kill the running command
 * - Release terminal resources
 *
 * **Important:** Always call `release()` when done with the terminal to free resources.

 * The terminal supports async disposal via `Symbol.asyncDispose` for automatic cleanup.

 * You can use `await using` to ensure the terminal is automatically released when it
 * goes out of scope.
 */
export class TerminalHandle {
  #sessionId: string;
  #connection: Connection;

  constructor(
    public id: string,
    sessionId: string,
    conn: Connection,
  ) {
    this.#sessionId = sessionId;
    this.#connection = conn;
  }

  /**
   * Gets the current terminal output without waiting for the command to exit.
   */
  async currentOutput(): Promise<schema.TerminalOutputResponse> {
    return await this.#connection.sendRequest(
      schema.CLIENT_METHODS.terminal_output,
      {
        sessionId: this.#sessionId,
        terminalId: this.id,
      },
    );
  }

  /**
   * Waits for the terminal command to complete and returns its exit status.
   */
  async waitForExit(): Promise<schema.WaitForTerminalExitResponse> {
    return await this.#connection.sendRequest(
      schema.CLIENT_METHODS.terminal_wait_for_exit,
      {
        sessionId: this.#sessionId,
        terminalId: this.id,
      },
    );
  }

  /**
   * Kills the terminal command without releasing the terminal.
   *
   * The terminal remains valid after killing, allowing you to:
   * - Get the final output with `currentOutput()`
   * - Check the exit status
   * - Release the terminal when done
   *
   * Useful for implementing timeouts or cancellation.
   */
  async kill(): Promise<schema.KillTerminalResponse> {
    return (
      (await this.#connection.sendRequest(schema.CLIENT_METHODS.terminal_kill, {
        sessionId: this.#sessionId,
        terminalId: this.id,
      })) ?? {}
    );
  }

  /**
   * Releases the terminal and frees all associated resources.
   *
   * If the command is still running, it will be killed.
   * After release, the terminal ID becomes invalid and cannot be used
   * with other terminal methods.
   *
   * Tool calls that already reference this terminal will continue to
   * display its output.
   *
   * **Important:** Always call this method when done with the terminal.
   */
  async release(): Promise<schema.ReleaseTerminalResponse | void> {
    return (
      (await this.#connection.sendRequest(
        schema.CLIENT_METHODS.terminal_release,
        {
          sessionId: this.#sessionId,
          terminalId: this.id,
        },
      )) ?? {}
    );
  }

  async [Symbol.asyncDispose](): Promise<void> {
    await this.release();
  }
}

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
export class ClientSideConnection implements Agent {
  #connection: Connection;

  /**
   * Creates a new client-side connection to an agent.
   *
   * This establishes the communication channel between a client and agent
   * following the ACP specification.
   *
   * @param toClient - A function that creates a Client handler to process incoming agent requests
   * @param stream - The bidirectional message stream for communication. Typically created using
   *                 {@link ndJsonStream} for stdio-based connections.
   *
   * See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)
   */
  constructor(toClient: (agent: Agent) => Client, stream: Stream) {
    const client = toClient(this);

    const requestHandler = async (
      method: string,
      params: unknown,
    ): Promise<unknown> => {
      switch (method) {
        case schema.CLIENT_METHODS.fs_write_text_file: {
          const validatedParams =
            schema.writeTextFileRequestSchema.parse(params);
          return client.writeTextFile(
            validatedParams as schema.WriteTextFileRequest,
          );
        }
        case schema.CLIENT_METHODS.fs_read_text_file: {
          const validatedParams =
            schema.readTextFileRequestSchema.parse(params);
          return client.readTextFile(
            validatedParams as schema.ReadTextFileRequest,
          );
        }
        case schema.CLIENT_METHODS.session_request_permission: {
          const validatedParams =
            schema.requestPermissionRequestSchema.parse(params);
          return client.requestPermission(
            validatedParams as schema.RequestPermissionRequest,
          );
        }
        case schema.CLIENT_METHODS.terminal_create: {
          const validatedParams =
            schema.createTerminalRequestSchema.parse(params);
          return client.createTerminal?.(
            validatedParams as schema.CreateTerminalRequest,
          );
        }
        case schema.CLIENT_METHODS.terminal_output: {
          const validatedParams =
            schema.terminalOutputRequestSchema.parse(params);
          return client.terminalOutput?.(
            validatedParams as schema.TerminalOutputRequest,
          );
        }
        case schema.CLIENT_METHODS.terminal_release: {
          const validatedParams =
            schema.releaseTerminalRequestSchema.parse(params);
          const result = await client.releaseTerminal?.(
            validatedParams as schema.ReleaseTerminalRequest,
          );
          return result ?? {};
        }
        case schema.CLIENT_METHODS.terminal_wait_for_exit: {
          const validatedParams =
            schema.waitForTerminalExitRequestSchema.parse(params);
          return client.waitForTerminalExit?.(
            validatedParams as schema.WaitForTerminalExitRequest,
          );
        }
        case schema.CLIENT_METHODS.terminal_kill: {
          const validatedParams =
            schema.killTerminalCommandRequestSchema.parse(params);
          const result = await client.killTerminal?.(
            validatedParams as schema.KillTerminalCommandRequest,
          );
          return result ?? {};
        }
        default:
          // Handle extension methods (any method starting with '_')
          if (method.startsWith("_")) {
            const customMethod = method.substring(1);
            if (!client.extMethod) {
              throw RequestError.methodNotFound(method);
            }
            return client.extMethod(
              customMethod,
              params as Record<string, unknown>,
            );
          }
          throw RequestError.methodNotFound(method);
      }
    };

    const notificationHandler = async (
      method: string,
      params: unknown,
    ): Promise<void> => {
      switch (method) {
        case schema.CLIENT_METHODS.session_update: {
          const validatedParams =
            schema.sessionNotificationSchema.parse(params);
          return client.sessionUpdate(
            validatedParams as schema.SessionNotification,
          );
        }
        default:
          // Handle extension notifications (any method starting with '_')
          if (method.startsWith("_")) {
            const customMethod = method.substring(1);
            if (!client.extNotification) {
              return;
            }
            return client.extNotification(
              customMethod,
              params as Record<string, unknown>,
            );
          }
          throw RequestError.methodNotFound(method);
      }
    };

    this.#connection = new Connection(
      requestHandler,
      notificationHandler,
      stream,
    );
  }

  /**
   * Establishes the connection with a client and negotiates protocol capabilities.
   *
   * This method is called once at the beginning of the connection to:
   * - Negotiate the protocol version to use
   * - Exchange capability information between client and agent
   * - Determine available authentication methods
   *
   * The agent should respond with its supported protocol version and capabilities.
   *
   * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
   */
  async initialize(
    params: schema.InitializeRequest,
  ): Promise<schema.InitializeResponse> {
    return await this.#connection.sendRequest(
      schema.AGENT_METHODS.initialize,
      params,
    );
  }

  /**
   * Creates a new conversation session with the agent.
   *
   * Sessions represent independent conversation contexts with their own history and state.
   *
   * The agent should:
   * - Create a new session context
   * - Connect to any specified MCP servers
   * - Return a unique session ID for future requests
   *
   * May return an `auth_required` error if the agent requires authentication.
   *
   * See protocol docs: [Session Setup](https://agentclientprotocol.com/protocol/session-setup)
   */
  async newSession(
    params: schema.NewSessionRequest,
  ): Promise<schema.NewSessionResponse> {
    return await this.#connection.sendRequest(
      schema.AGENT_METHODS.session_new,
      params,
    );
  }

  /**
   * Loads an existing session to resume a previous conversation.
   *
   * This method is only available if the agent advertises the `loadSession` capability.
   *
   * The agent should:
   * - Restore the session context and conversation history
   * - Connect to the specified MCP servers
   * - Stream the entire conversation history back to the client via notifications
   *
   * See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
   */
  async loadSession(
    params: schema.LoadSessionRequest,
  ): Promise<schema.LoadSessionResponse> {
    return (
      (await this.#connection.sendRequest(
        schema.AGENT_METHODS.session_load,
        params,
      )) ?? {}
    );
  }

  /**
   * Sets the operational mode for a session.
   *
   * Allows switching between different agent modes (e.g., "ask", "architect", "code")
   * that affect system prompts, tool availability, and permission behaviors.
   *
   * The mode must be one of the modes advertised in `availableModes` during session
   * creation or loading. Agents may also change modes autonomously and notify the
   * client via `current_mode_update` notifications.
   *
   * This method can be called at any time during a session, whether the Agent is
   * idle or actively generating a turn.
   *
   * See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
   */
  async setSessionMode(
    params: schema.SetSessionModeRequest,
  ): Promise<schema.SetSessionModeResponse> {
    return (
      (await this.#connection.sendRequest(
        schema.AGENT_METHODS.session_set_mode,
        params,
      )) ?? {}
    );
  }

  /**
   * Authenticates the client using the specified authentication method.
   *
   * Called when the agent requires authentication before allowing session creation.
   * The client provides the authentication method ID that was advertised during initialization.
   *
   * After successful authentication, the client can proceed to create sessions with
   * `newSession` without receiving an `auth_required` error.
   *
   * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
   */
  async authenticate(
    params: schema.AuthenticateRequest,
  ): Promise<schema.AuthenticateResponse> {
    return (
      (await this.#connection.sendRequest(
        schema.AGENT_METHODS.authenticate,
        params,
      )) ?? {}
    );
  }

  /**
   * Processes a user prompt within a session.
   *
   * This method handles the whole lifecycle of a prompt:
   * - Receives user messages with optional context (files, images, etc.)
   * - Processes the prompt using language models
   * - Reports language model content and tool calls to the Clients
   * - Requests permission to run tools
   * - Executes any requested tool calls
   * - Returns when the turn is complete with a stop reason
   *
   * See protocol docs: [Prompt Turn](https://agentclientprotocol.com/protocol/prompt-turn)
   */
  async prompt(params: schema.PromptRequest): Promise<schema.PromptResponse> {
    return await this.#connection.sendRequest(
      schema.AGENT_METHODS.session_prompt,
      params,
    );
  }

  /**
   * Cancels ongoing operations for a session.
   *
   * This is a notification sent by the client to cancel an ongoing prompt turn.
   *
   * Upon receiving this notification, the Agent SHOULD:
   * - Stop all language model requests as soon as possible
   * - Abort all tool call invocations in progress
   * - Send any pending `session/update` notifications
   * - Respond to the original `session/prompt` request with `StopReason::Cancelled`
   *
   * See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
   */
  async cancel(params: schema.CancelNotification): Promise<void> {
    return await this.#connection.sendNotification(
      schema.AGENT_METHODS.session_cancel,
      params,
    );
  }

  /**
   * Extension method
   *
   * Allows the Client to send an arbitrary request that is not part of the ACP spec.
   */
  async extMethod(
    method: string,
    params: Record<string, unknown>,
  ): Promise<Record<string, unknown>> {
    return await this.#connection.sendRequest(`_${method}`, params);
  }

  /**
   * Extension notification
   *
   * Allows the Client to send an arbitrary notification that is not part of the ACP spec.
   */
  async extNotification(
    method: string,
    params: Record<string, unknown>,
  ): Promise<void> {
    return await this.#connection.sendNotification(`_${method}`, params);
  }
}

// Re-export AnyMessage for backwards compatibility
export type { AnyMessage } from "./jsonrpc.js";

class Connection {
  #pendingResponses: Map<string | number, PendingResponse> = new Map();
  #nextRequestId: number = 0;
  #requestHandler: RequestHandler;
  #notificationHandler: NotificationHandler;
  #stream: Stream;
  #writeQueue: Promise<void> = Promise.resolve();

  constructor(
    requestHandler: RequestHandler,
    notificationHandler: NotificationHandler,
    stream: Stream,
  ) {
    this.#requestHandler = requestHandler;
    this.#notificationHandler = notificationHandler;
    this.#stream = stream;
    this.#receive();
  }

  async #receive() {
    const reader = this.#stream.readable.getReader();
    try {
      while (true) {
        const { value: message, done } = await reader.read();
        if (done) {
          break;
        }
        if (!message) {
          continue;
        }

        try {
          this.#processMessage(message);
        } catch (err) {
          console.error(
            "Unexpected error during message processing:",
            message,
            err,
          );
          // Only send error response if the message had an id (was a request)
          if ("id" in message && message.id !== undefined) {
            this.#sendMessage({
              jsonrpc: "2.0",
              id: message.id,
              error: {
                code: -32700,
                message: "Parse error",
              },
            });
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
      const response = await this.#tryCallRequestHandler(
        message.method,
        message.params,
      );
      if ("error" in response) {
        console.error("Error handling request", message, response.error);
      }

      await this.#sendMessage({
        jsonrpc: "2.0",
        id: message.id,
        ...response,
      });
    } else if ("method" in message) {
      // It's a notification
      const response = await this.#tryCallNotificationHandler(
        message.method,
        message.params,
      );
      if ("error" in response) {
        console.error("Error handling notification", message, response.error);
      }
    } else if ("id" in message) {
      // It's a response
      this.#handleResponse(message as AnyResponse);
    } else {
      console.error("Invalid message", { message });
    }
  }

  async #tryCallRequestHandler(
    method: string,
    params: unknown,
  ): Promise<Result<unknown>> {
    try {
      const result = await this.#requestHandler(method, params);
      return { result: result ?? null };
    } catch (error: unknown) {
      if (error instanceof RequestError) {
        return error.toResult();
      }

      if (error instanceof z.ZodError) {
        return RequestError.invalidParams(error.format()).toResult();
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

      try {
        return RequestError.internalError(
          details ? JSON.parse(details) : {},
        ).toResult();
      } catch (_err) {
        return RequestError.internalError({ details }).toResult();
      }
    }
  }

  async #tryCallNotificationHandler(
    method: string,
    params: unknown,
  ): Promise<Result<unknown>> {
    try {
      await this.#notificationHandler(method, params);
      return { result: null };
    } catch (error: unknown) {
      if (error instanceof RequestError) {
        return error.toResult();
      }

      if (error instanceof z.ZodError) {
        return RequestError.invalidParams(error.format()).toResult();
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

      try {
        return RequestError.internalError(
          details ? JSON.parse(details) : {},
        ).toResult();
      } catch (_err) {
        return RequestError.internalError({ details }).toResult();
      }
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
    } else {
      console.error("Got response to unknown request", response.id);
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

  async #sendMessage(message: AnyMessage) {
    this.#writeQueue = this.#writeQueue
      .then(async () => {
        const writer = this.#stream.writable.getWriter();
        try {
          await writer.write(message);
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

/**
 * JSON-RPC error object.
 *
 * Represents an error that occurred during method execution, following the
 * JSON-RPC 2.0 error object specification with optional additional data.
 *
 * See protocol docs: [JSON-RPC Error Object](https://www.jsonrpc.org/specification#error_object)
 */
export class RequestError extends Error {
  data?: unknown;

  constructor(
    public code: number,
    message: string,
    data?: unknown,
  ) {
    super(message);
    this.name = "RequestError";
    this.data = data;
  }

  /**
   * Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
   */
  static parseError(data?: object): RequestError {
    return new RequestError(-32700, "Parse error", data);
  }

  /**
   * The JSON sent is not a valid Request object.
   */
  static invalidRequest(data?: object): RequestError {
    return new RequestError(-32600, "Invalid request", data);
  }

  /**
   * The method does not exist / is not available.
   */
  static methodNotFound(method: string): RequestError {
    return new RequestError(-32601, "Method not found", { method });
  }

  /**
   * Invalid method parameter(s).
   */
  static invalidParams(data?: object): RequestError {
    return new RequestError(-32602, "Invalid params", data);
  }

  /**
   * Internal JSON-RPC error.
   */
  static internalError(data?: object): RequestError {
    return new RequestError(-32603, "Internal error", data);
  }

  /**
   * Authentication required.
   */
  static authRequired(data?: object): RequestError {
    return new RequestError(-32000, "Authentication required", data);
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

  toErrorResponse(): ErrorResponse {
    return {
      code: this.code,
      message: this.message,
      data: this.data,
    };
  }
}

/**
 * The Client interface defines the interface that ACP-compliant clients must implement.
 *
 * Clients are typically code editors (IDEs, text editors) that provide the interface
 * between users and AI agents. They manage the environment, handle user interactions,
 * and control access to resources.
 */
export interface Client {
  /**
   * Requests permission from the user for a tool call operation.
   *
   * Called by the agent when it needs user authorization before executing
   * a potentially sensitive operation. The client should present the options
   * to the user and return their decision.
   *
   * If the client cancels the prompt turn via `session/cancel`, it MUST
   * respond to this request with `RequestPermissionOutcome::Cancelled`.
   *
   * See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
   */
  requestPermission(
    params: schema.RequestPermissionRequest,
  ): Promise<schema.RequestPermissionResponse>;
  /**
   * Handles session update notifications from the agent.
   *
   * This is a notification endpoint (no response expected) that receives
   * real-time updates about session progress, including message chunks,
   * tool calls, and execution plans.
   *
   * Note: Clients SHOULD continue accepting tool call updates even after
   * sending a `session/cancel` notification, as the agent may send final
   * updates before responding with the cancelled stop reason.
   *
   * See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
   */
  sessionUpdate(params: schema.SessionNotification): Promise<void>;
  /**
   * Writes content to a text file in the client's file system.
   *
   * Only available if the client advertises the `fs.writeTextFile` capability.
   * Allows the agent to create or modify files within the client's environment.
   *
   * See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
   */
  writeTextFile(
    params: schema.WriteTextFileRequest,
  ): Promise<schema.WriteTextFileResponse>;
  /**
   * Reads content from a text file in the client's file system.
   *
   * Only available if the client advertises the `fs.readTextFile` capability.
   * Allows the agent to access file contents within the client's environment.
   *
   * See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
   */
  readTextFile(
    params: schema.ReadTextFileRequest,
  ): Promise<schema.ReadTextFileResponse>;

  /**
   * Creates a new terminal to execute a command.
   *
   * Only available if the `terminal` capability is set to `true`.
   *
   * The Agent must call `releaseTerminal` when done with the terminal
   * to free resources.

   * @see {@link https://agentclientprotocol.com/protocol/terminals | Terminal Documentation}
   */
  createTerminal?(
    params: schema.CreateTerminalRequest,
  ): Promise<schema.CreateTerminalResponse>;

  /**
   * Gets the current output and exit status of a terminal.
   *
   * Returns immediately without waiting for the command to complete.
   * If the command has already exited, the exit status is included.
   *
   * @see {@link https://agentclientprotocol.com/protocol/terminals#getting-output | Getting Terminal Output}
   */
  terminalOutput?(
    params: schema.TerminalOutputRequest,
  ): Promise<schema.TerminalOutputResponse>;

  /**
   * Releases a terminal and frees all associated resources.
   *
   * The command is killed if it hasn't exited yet. After release,
   * the terminal ID becomes invalid for all other terminal methods.
   *
   * Tool calls that already contain the terminal ID continue to
   * display its output.
   *
   * @see {@link https://agentclientprotocol.com/protocol/terminals#releasing-terminals | Releasing Terminals}
   */
  releaseTerminal?(
    params: schema.ReleaseTerminalRequest,
  ): Promise<schema.ReleaseTerminalResponse | void>;

  /**
   * Waits for a terminal command to exit and returns its exit status.
   *
   * This method returns once the command completes, providing the
   * exit code and/or signal that terminated the process.
   *
   * @see {@link https://agentclientprotocol.com/protocol/terminals#waiting-for-exit | Waiting for Exit}
   */
  waitForTerminalExit?(
    params: schema.WaitForTerminalExitRequest,
  ): Promise<schema.WaitForTerminalExitResponse>;

  /**
   * Kills a terminal command without releasing the terminal.
   *
   * While `releaseTerminal` also kills the command, this method keeps
   * the terminal ID valid so it can be used with other methods.
   *
   * Useful for implementing command timeouts that terminate the command
   * and then retrieve the final output.
   *
   * Note: Call `releaseTerminal` when the terminal is no longer needed.
   *
   * @see {@link https://agentclientprotocol.com/protocol/terminals#killing-commands | Killing Commands}
   */
  killTerminal?(
    params: schema.KillTerminalCommandRequest,
  ): Promise<schema.KillTerminalResponse | void>;

  /**
   * Extension method
   *
   * Allows the Agent to send an arbitrary request that is not part of the ACP spec.
   *
   * To help avoid conflicts, it's a good practice to prefix extension
   * methods with a unique identifier such as domain name.
   */
  extMethod?(
    method: string,
    params: Record<string, unknown>,
  ): Promise<Record<string, unknown>>;

  /**
   * Extension notification
   *
   * Allows the Agent to send an arbitrary notification that is not part of the ACP spec.
   */
  extNotification?(
    method: string,
    params: Record<string, unknown>,
  ): Promise<void>;
}

/**
 * The Agent interface defines the interface that all ACP-compliant agents must implement.
 *
 * Agents are programs that use generative AI to autonomously modify code. They handle
 * requests from clients and execute tasks using language models and tools.
 */
export interface Agent {
  /**
   * Establishes the connection with a client and negotiates protocol capabilities.
   *
   * This method is called once at the beginning of the connection to:
   * - Negotiate the protocol version to use
   * - Exchange capability information between client and agent
   * - Determine available authentication methods
   *
   * The agent should respond with its supported protocol version and capabilities.
   *
   * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
   */
  initialize(
    params: schema.InitializeRequest,
  ): Promise<schema.InitializeResponse>;
  /**
   * Creates a new conversation session with the agent.
   *
   * Sessions represent independent conversation contexts with their own history and state.
   *
   * The agent should:
   * - Create a new session context
   * - Connect to any specified MCP servers
   * - Return a unique session ID for future requests
   *
   * May return an `auth_required` error if the agent requires authentication.
   *
   * See protocol docs: [Session Setup](https://agentclientprotocol.com/protocol/session-setup)
   */
  newSession(
    params: schema.NewSessionRequest,
  ): Promise<schema.NewSessionResponse>;
  /**
   * Loads an existing session to resume a previous conversation.
   *
   * This method is only available if the agent advertises the `loadSession` capability.
   *
   * The agent should:
   * - Restore the session context and conversation history
   * - Connect to the specified MCP servers
   * - Stream the entire conversation history back to the client via notifications
   *
   * See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
   */
  loadSession?(
    params: schema.LoadSessionRequest,
  ): Promise<schema.LoadSessionResponse>;
  /**
   * Sets the operational mode for a session.
   *
   * Allows switching between different agent modes (e.g., "ask", "architect", "code")
   * that affect system prompts, tool availability, and permission behaviors.
   *
   * The mode must be one of the modes advertised in `availableModes` during session
   * creation or loading. Agents may also change modes autonomously and notify the
   * client via `current_mode_update` notifications.
   *
   * This method can be called at any time during a session, whether the Agent is
   * idle or actively generating a turn.
   *
   * See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
   */
  setSessionMode?(
    params: schema.SetSessionModeRequest,
  ): Promise<schema.SetSessionModeResponse | void>;
  /**
   * Authenticates the client using the specified authentication method.
   *
   * Called when the agent requires authentication before allowing session creation.
   * The client provides the authentication method ID that was advertised during initialization.
   *
   * After successful authentication, the client can proceed to create sessions with
   * `newSession` without receiving an `auth_required` error.
   *
   * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
   */
  authenticate(
    params: schema.AuthenticateRequest,
  ): Promise<schema.AuthenticateResponse | void>;
  /**
   * Processes a user prompt within a session.
   *
   * This method handles the whole lifecycle of a prompt:
   * - Receives user messages with optional context (files, images, etc.)
   * - Processes the prompt using language models
   * - Reports language model content and tool calls to the Clients
   * - Requests permission to run tools
   * - Executes any requested tool calls
   * - Returns when the turn is complete with a stop reason
   *
   * See protocol docs: [Prompt Turn](https://agentclientprotocol.com/protocol/prompt-turn)
   */
  prompt(params: schema.PromptRequest): Promise<schema.PromptResponse>;
  /**
   * Cancels ongoing operations for a session.
   *
   * This is a notification sent by the client to cancel an ongoing prompt turn.
   *
   * Upon receiving this notification, the Agent SHOULD:
   * - Stop all language model requests as soon as possible
   * - Abort all tool call invocations in progress
   * - Send any pending `session/update` notifications
   * - Respond to the original `session/prompt` request with `StopReason::Cancelled`
   *
   * See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
   */
  cancel(params: schema.CancelNotification): Promise<void>;

  /**
   * Extension method
   *
   * Allows the Client to send an arbitrary request that is not part of the ACP spec.
   *
   * To help avoid conflicts, it's a good practice to prefix extension
   * methods with a unique identifier such as domain name.
   */
  extMethod?(
    method: string,
    params: Record<string, unknown>,
  ): Promise<Record<string, unknown>>;

  /**
   * Extension notification
   *
   * Allows the Client to send an arbitrary notification that is not part of the ACP spec.
   */
  extNotification?(
    method: string,
    params: Record<string, unknown>,
  ): Promise<void>;
}
