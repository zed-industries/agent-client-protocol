export type AgentCodingProtocol =
  | AnyClientRequest
  | AnyClientResult
  | AnyAgentRequest
  | AnyAgentResult;
export type AnyClientRequest =
  | StreamAssistantMessageChunkParams
  | RequestToolCallConfirmationParams
  | PushToolCallParams
  | UpdateToolCallParams;
export type AssistantMessageChunk =
  | {
      text: string;
    }
  | {
      thought: string;
    };
export type ToolCallConfirmation =
  | {
      description?: string | null;
      type: "edit";
    }
  | {
      description?: string | null;
      type: "execute";
      command: string;
      rootCommand: string;
    }
  | {
      description?: string | null;
      type: "mcp";
      serverName: string;
      toolDisplayName: string;
      toolName: string;
    }
  | {
      description?: string | null;
      type: "fetch";
      urls: string[];
    }
  | {
      description: string;
      type: "other";
    };
export type ToolCallContent =
  | {
      type: "markdown";
      markdown: string;
    }
  | {
      type: "diff";
      newText: string;
      oldText: string | null;
      path: string;
    };
export type Icon =
  | "fileSearch"
  | "folder"
  | "globe"
  | "hammer"
  | "lightBulb"
  | "pencil"
  | "regex"
  | "terminal";
export type ToolCallStatus = "running" | "finished" | "error";
export type ToolCallId = number;
export type AnyClientResult =
  | StreamAssistantMessageChunkResponse
  | RequestToolCallConfirmationResponse
  | PushToolCallResponse
  | UpdateToolCallResponse;
export type StreamAssistantMessageChunkResponse = null;
export type ToolCallConfirmationOutcome =
  | "allow"
  | "alwaysAllow"
  | "alwaysAllowMcpServer"
  | "alwaysAllowTool"
  | "reject"
  | "cancel";
export type UpdateToolCallResponse = null;
export type AnyAgentRequest =
  | InitializeParams
  | AuthenticateParams
  | SendUserMessageParams
  | CancelSendMessageParams;
/**
 * Initialize sets up the agent's state. It should be called before any other method,
 * and no other methods should be called until it has completed.
 *
 * If the agent is not authenticated, then the client should prompt the user to authenticate,
 * and then call the `authenticate` method.
 * Otherwise the client can send other messages to the agent.
 */
export type InitializeParams = null;
/**
 * Triggers authentication on the agent side.
 *
 * This method should only be called if the initialize response indicates the user isn't already authenticated.
 * If this succceeds then the client can send other messasges to the agent,
 * If it fails then the error message should be shown and the user prompted to authenticate.
 *
 * The implementation of authentication is left up to the agent, typically an oauth
 * flow is run by opening a browser window in the background.
 */
export type AuthenticateParams = null;
/**
 * A part in a user message
 */
export type UserMessageChunk =
  | {
      text: string;
    }
  | {
      path: string;
    };
/**
 * cancelSendMessage allows the client to request that the agent
 * stop running. The agent should resolve or reject the current sendUserMessage call.
 */
export type CancelSendMessageParams = null;
export type AnyAgentResult =
  | InitializeResponse
  | AuthenticateResponse
  | SendUserMessageResponse
  | CancelSendMessageResponse;
export type AuthenticateResponse = null;
export type SendUserMessageResponse = null;
export type CancelSendMessageResponse = null;

/**
 * Streams part of an assistant response to the client
 */
export interface StreamAssistantMessageChunkParams {
  chunk: AssistantMessageChunk;
}
/**
 * Request confirmation before running a tool
 *
 * When allowed, the client returns a [`ToolCallId`] which can be used
 * to update the tool call's `status` and `content` as it runs.
 */
export interface RequestToolCallConfirmationParams {
  confirmation: ToolCallConfirmation;
  content?: ToolCallContent | null;
  icon: Icon;
  label: string;
  locations?: ToolCallLocation[];
}
export interface ToolCallLocation {
  line?: number | null;
  path: string;
}
/**
 * pushToolCall allows the agent to start a tool call
 * when it does not need to request permission to do so.
 *
 * The returned id can be used to update the UI for the tool
 * call as needed.
 */
export interface PushToolCallParams {
  content?: ToolCallContent | null;
  icon: Icon;
  label: string;
  locations?: ToolCallLocation[];
}
/**
 * updateToolCall allows the agent to update the content and status of the tool call.
 *
 * The new content replaces what is currently displayed in the UI.
 *
 * The [`ToolCallId`] is included in the response of
 * `pushToolCall` or `requestToolCallConfirmation` respectively.
 */
export interface UpdateToolCallParams {
  content: ToolCallContent | null;
  status: ToolCallStatus;
  toolCallId: ToolCallId;
}
export interface RequestToolCallConfirmationResponse {
  id: ToolCallId;
  outcome: ToolCallConfirmationOutcome;
}
export interface PushToolCallResponse {
  id: ToolCallId;
}
/**
 * sendUserMessage allows the user to send a message to the agent.
 * This method should complete after the agent is finished, during
 * which time the agent may update the client by calling
 * streamAssistantMessageChunk and other methods.
 */
export interface SendUserMessageParams {
  chunks: UserMessageChunk[];
}
export interface InitializeResponse {
  /**
   * Indicates whether the agent is authenticated and
   * ready to handle requests.
   */
  isAuthenticated: boolean;
}

export interface Method {
  name: string;
  requestType: string;
  paramPayload: boolean;
  responseType: string;
  responsePayload: boolean;
}

export interface Client {
  streamAssistantMessageChunk(
    params: StreamAssistantMessageChunkParams,
  ): Promise<void>;
  requestToolCallConfirmation(
    params: RequestToolCallConfirmationParams,
  ): Promise<RequestToolCallConfirmationResponse>;
  pushToolCall(params: PushToolCallParams): Promise<PushToolCallResponse>;
  updateToolCall(params: UpdateToolCallParams): Promise<void>;
}

export const CLIENT_METHODS: Method[] = [
  {
    name: "streamAssistantMessageChunk",
    requestType: "StreamAssistantMessageChunkParams",
    paramPayload: true,
    responseType: "StreamAssistantMessageChunkResponse",
    responsePayload: false,
  },
  {
    name: "requestToolCallConfirmation",
    requestType: "RequestToolCallConfirmationParams",
    paramPayload: true,
    responseType: "RequestToolCallConfirmationResponse",
    responsePayload: true,
  },
  {
    name: "pushToolCall",
    requestType: "PushToolCallParams",
    paramPayload: true,
    responseType: "PushToolCallResponse",
    responsePayload: true,
  },
  {
    name: "updateToolCall",
    requestType: "UpdateToolCallParams",
    paramPayload: true,
    responseType: "UpdateToolCallResponse",
    responsePayload: false,
  },
];

export interface Agent {
  initialize(): Promise<InitializeResponse>;
  authenticate(): Promise<void>;
  sendUserMessage(params: SendUserMessageParams): Promise<void>;
  cancelSendMessage(): Promise<void>;
}

export const AGENT_METHODS: Method[] = [
  {
    name: "initialize",
    requestType: "InitializeParams",
    paramPayload: false,
    responseType: "InitializeResponse",
    responsePayload: true,
  },
  {
    name: "authenticate",
    requestType: "AuthenticateParams",
    paramPayload: false,
    responseType: "AuthenticateResponse",
    responsePayload: false,
  },
  {
    name: "sendUserMessage",
    requestType: "SendUserMessageParams",
    paramPayload: true,
    responseType: "SendUserMessageResponse",
    responsePayload: false,
  },
  {
    name: "cancelSendMessage",
    requestType: "CancelSendMessageParams",
    paramPayload: false,
    responseType: "CancelSendMessageResponse",
    responsePayload: false,
  },
];
