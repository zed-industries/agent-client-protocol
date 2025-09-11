export const AGENT_METHODS = {
  authenticate: "authenticate",
  ext_method: "_method",
  ext_notification: "_notification",
  initialize: "initialize",
  session_cancel: "session/cancel",
  session_load: "session/load",
  session_new: "session/new",
  session_prompt: "session/prompt",
  session_set_mode: "session/set_mode",
};

export const CLIENT_METHODS = {
  ext_method: "_method",
  ext_notification: "_notification",
  fs_read_text_file: "fs/read_text_file",
  fs_write_text_file: "fs/write_text_file",
  session_request_permission: "session/request_permission",
  session_update: "session/update",
  terminal_create: "terminal/create",
  terminal_kill: "terminal/kill",
  terminal_output: "terminal/output",
  terminal_release: "terminal/release",
  terminal_wait_for_exit: "terminal/wait_for_exit",
};

export const PROTOCOL_VERSION = 1;

import { z } from "zod";

export type AgentClientProtocol =
  | ClientRequest
  | ClientResponse
  | ClientNotification
  | AgentRequest
  | AgentResponse
  | AgentNotification;
/**
 * All possible requests that an agent can send to a client.
 *
 * This enum is used internally for routing RPC requests. You typically won't need
 * to use this directly - instead, use the methods on the [`Client`] trait.
 *
 * This enum encompasses all method calls from agent to client.
 */
/** @internal */
export type ClientRequest =
  | WriteTextFileRequest
  | ReadTextFileRequest
  | RequestPermissionRequest
  | CreateTerminalRequest
  | TerminalOutputRequest
  | ReleaseTerminalRequest
  | WaitForTerminalExitRequest
  | KillTerminalRequest
  | ExtMethodRequest;
/**
 * Content produced by a tool call.
 *
 * Tool calls can produce different types of content including
 * standard content blocks (text, images) or file diffs.
 *
 * See protocol docs: [Content](https://agentclientprotocol.com/protocol/tool-calls#content)
 */
export type ToolCallContent =
  | {
      /**
       * Content blocks represent displayable information in the Agent Client Protocol.
       *
       * They provide a structured way to handle various types of user-facing content—whether
       * it's text from language models, images for analysis, or embedded resources for context.
       *
       * Content blocks appear in:
       * - User prompts sent via `session/prompt`
       * - Language model output streamed through `session/update` notifications
       * - Progress updates and results from tool calls
       *
       * This structure is compatible with the Model Context Protocol (MCP), enabling
       * agents to seamlessly forward content from MCP tool outputs without transformation.
       *
       * See protocol docs: [Content](https://agentclientprotocol.com/protocol/content)
       */
      content:
        | {
            /**
             * Extension point for implementations
             */
            _meta?: {
              [k: string]: unknown;
            };
            annotations?: Annotations | null;
            text: string;
            type: "text";
          }
        | {
            /**
             * Extension point for implementations
             */
            _meta?: {
              [k: string]: unknown;
            };
            annotations?: Annotations | null;
            data: string;
            mimeType: string;
            type: "image";
            uri?: string | null;
          }
        | {
            /**
             * Extension point for implementations
             */
            _meta?: {
              [k: string]: unknown;
            };
            annotations?: Annotations | null;
            data: string;
            mimeType: string;
            type: "audio";
          }
        | {
            /**
             * Extension point for implementations
             */
            _meta?: {
              [k: string]: unknown;
            };
            annotations?: Annotations | null;
            description?: string | null;
            mimeType?: string | null;
            name: string;
            size?: number | null;
            title?: string | null;
            type: "resource_link";
            uri: string;
          }
        | {
            /**
             * Extension point for implementations
             */
            _meta?: {
              [k: string]: unknown;
            };
            annotations?: Annotations | null;
            resource: EmbeddedResourceResource;
            type: "resource";
          };
      type: "content";
    }
  | {
      /**
       * Extension point for implementations
       */
      _meta?: {
        [k: string]: unknown;
      };
      /**
       * The new content after modification.
       */
      newText: string;
      /**
       * The original content (None for new files).
       */
      oldText?: string | null;
      /**
       * The file path being modified.
       */
      path: string;
      type: "diff";
    }
  | {
      terminalId: string;
      type: "terminal";
    };
/**
 * The sender or recipient of messages and data in a conversation.
 */
export type Role = "assistant" | "user";
/**
 * Resource content that can be embedded in a message.
 */
export type EmbeddedResourceResource =
  | TextResourceContents
  | BlobResourceContents;
/**
 * Categories of tools that can be invoked.
 *
 * Tool kinds help clients choose appropriate icons and optimize how they
 * display tool execution progress.
 *
 * See protocol docs: [Creating](https://agentclientprotocol.com/protocol/tool-calls#creating)
 */
export type ToolKind =
  | "read"
  | "edit"
  | "delete"
  | "move"
  | "search"
  | "execute"
  | "think"
  | "fetch"
  | "switch_mode"
  | "other";
/**
 * Execution status of a tool call.
 *
 * Tool calls progress through different statuses during their lifecycle.
 *
 * See protocol docs: [Status](https://agentclientprotocol.com/protocol/tool-calls#status)
 */
export type ToolCallStatus = "pending" | "in_progress" | "completed" | "failed";
/**
 * A unique identifier for a conversation session between a client and agent.
 *
 * Sessions maintain their own context, conversation history, and state,
 * allowing multiple independent interactions with the same agent.
 *
 * # Example
 *
 * ```
 * use agent_client_protocol::SessionId;
 * use std::sync::Arc;
 *
 * let session_id = SessionId(Arc::from("sess_abc123def456"));
 * ```
 *
 * See protocol docs: [Session ID](https://agentclientprotocol.com/protocol/session-setup#session-id)
 */
export type SessionId = string;
/**
 * All possible responses that a client can send to an agent.
 *
 * This enum is used internally for routing RPC responses. You typically won't need
 * to use this directly - the responses are handled automatically by the connection.
 *
 * These are responses to the corresponding AgentRequest variants.
 */
/** @internal */
export type ClientResponse =
  | WriteTextFileResponse
  | ReadTextFileResponse
  | RequestPermissionResponse
  | CreateTerminalResponse
  | TerminalOutputResponse
  | ReleaseTerminalResponse
  | WaitForTerminalExitResponse
  | KillTerminalResponse
  | ExtMethodResponse;
export type WriteTextFileResponse = null;
export type ReleaseTerminalResponse = null;
export type KillTerminalResponse = null;
/**
 * All possible notifications that a client can send to an agent.
 *
 * This enum is used internally for routing RPC notifications. You typically won't need
 * to use this directly - use the notification methods on the [`Agent`] trait instead.
 *
 * Notifications do not expect a response.
 */
/** @internal */
export type ClientNotification = CancelNotification | ExtNotification;
/**
 * All possible requests that a client can send to an agent.
 *
 * This enum is used internally for routing RPC requests. You typically won't need
 * to use this directly - instead, use the methods on the [`Agent`] trait.
 *
 * This enum encompasses all method calls from client to agent.
 */
/** @internal */
export type AgentRequest =
  | InitializeRequest
  | AuthenticateRequest
  | NewSessionRequest
  | LoadSessionRequest
  | SetSessionModeRequest
  | PromptRequest
  | ExtMethodRequest1;
/**
 * Configuration for connecting to an MCP (Model Context Protocol) server.
 *
 * MCP servers provide tools and context that the agent can use when
 * processing prompts.
 *
 * See protocol docs: [MCP Servers](https://agentclientprotocol.com/protocol/session-setup#mcp-servers)
 */
export type McpServer =
  | {
      /**
       * HTTP headers to set when making requests to the MCP server.
       */
      headers: HttpHeader[];
      /**
       * Human-readable name identifying this MCP server.
       */
      name: string;
      type: "http";
      /**
       * URL to the MCP server.
       */
      url: string;
    }
  | {
      /**
       * HTTP headers to set when making requests to the MCP server.
       */
      headers: HttpHeader[];
      /**
       * Human-readable name identifying this MCP server.
       */
      name: string;
      type: "sse";
      /**
       * URL to the MCP server.
       */
      url: string;
    }
  | Stdio;
/**
 * **UNSTABLE**
 *
 * This type is not part of the spec, and may be removed or changed at any point.
 */
export type SessionModeId = string;
/**
 * Content blocks represent displayable information in the Agent Client Protocol.
 *
 * They provide a structured way to handle various types of user-facing content—whether
 * it's text from language models, images for analysis, or embedded resources for context.
 *
 * Content blocks appear in:
 * - User prompts sent via `session/prompt`
 * - Language model output streamed through `session/update` notifications
 * - Progress updates and results from tool calls
 *
 * This structure is compatible with the Model Context Protocol (MCP), enabling
 * agents to seamlessly forward content from MCP tool outputs without transformation.
 *
 * See protocol docs: [Content](https://agentclientprotocol.com/protocol/content)
 */
export type ContentBlock =
  | {
      /**
       * Extension point for implementations
       */
      _meta?: {
        [k: string]: unknown;
      };
      annotations?: Annotations | null;
      text: string;
      type: "text";
    }
  | {
      /**
       * Extension point for implementations
       */
      _meta?: {
        [k: string]: unknown;
      };
      annotations?: Annotations | null;
      data: string;
      mimeType: string;
      type: "image";
      uri?: string | null;
    }
  | {
      /**
       * Extension point for implementations
       */
      _meta?: {
        [k: string]: unknown;
      };
      annotations?: Annotations | null;
      data: string;
      mimeType: string;
      type: "audio";
    }
  | {
      /**
       * Extension point for implementations
       */
      _meta?: {
        [k: string]: unknown;
      };
      annotations?: Annotations | null;
      description?: string | null;
      mimeType?: string | null;
      name: string;
      size?: number | null;
      title?: string | null;
      type: "resource_link";
      uri: string;
    }
  | {
      /**
       * Extension point for implementations
       */
      _meta?: {
        [k: string]: unknown;
      };
      annotations?: Annotations | null;
      resource: EmbeddedResourceResource;
      type: "resource";
    };
/**
 * All possible responses that an agent can send to a client.
 *
 * This enum is used internally for routing RPC responses. You typically won't need
 * to use this directly - the responses are handled automatically by the connection.
 *
 * These are responses to the corresponding ClientRequest variants.
 */
/** @internal */
export type AgentResponse =
  | InitializeResponse
  | AuthenticateResponse
  | NewSessionResponse
  | LoadSessionResponse
  | SetSessionModeResponse
  | PromptResponse
  | ExtMethodResponse1;
export type AuthenticateResponse = null;
/**
 * All possible notifications that an agent can send to a client.
 *
 * This enum is used internally for routing RPC notifications. You typically won't need
 * to use this directly - use the notification methods on the [`Client`] trait instead.
 *
 * Notifications do not expect a response.
 */
/** @internal */
export type AgentNotification = SessionNotification | ExtNotification1;
export type AvailableCommandInput = UnstructuredCommandInput;

/**
 * Request to write content to a text file.
 *
 * Only available if the client supports the `fs.writeTextFile` capability.
 */
export interface WriteTextFileRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * The text content to write to the file.
   */
  content: string;
  /**
   * Absolute path to the file to write.
   */
  path: string;
  /**
   * The session ID for this request.
   */
  sessionId: string;
}
/**
 * Request to read content from a text file.
 *
 * Only available if the client supports the `fs.readTextFile` capability.
 */
export interface ReadTextFileRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Optional maximum number of lines to read.
   */
  limit?: number | null;
  /**
   * Optional line number to start reading from (1-based).
   */
  line?: number | null;
  /**
   * Absolute path to the file to read.
   */
  path: string;
  /**
   * The session ID for this request.
   */
  sessionId: string;
}
/**
 * Request for user permission to execute a tool call.
 *
 * Sent when the agent needs authorization before performing a sensitive operation.
 *
 * See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
 */
export interface RequestPermissionRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Available permission options for the user to choose from.
   */
  options: PermissionOption[];
  /**
   * The session ID for this request.
   */
  sessionId: string;
  toolCall: ToolCallUpdate;
}
/**
 * An option presented to the user when requesting permission.
 */
export interface PermissionOption {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Hint about the nature of this permission option.
   */
  kind: "allow_once" | "allow_always" | "reject_once" | "reject_always";
  /**
   * Human-readable label to display to the user.
   */
  name: string;
  /**
   * Unique identifier for this permission option.
   */
  optionId: string;
}
/**
 * Details about the tool call requiring permission.
 */
export interface ToolCallUpdate {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Replace the content collection.
   */
  content?: ToolCallContent[] | null;
  /**
   * Update the tool kind.
   */
  kind?: ToolKind | null;
  /**
   * Replace the locations collection.
   */
  locations?: ToolCallLocation[] | null;
  /**
   * Update the raw input.
   */
  rawInput?: {
    [k: string]: unknown;
  };
  /**
   * Update the raw output.
   */
  rawOutput?: {
    [k: string]: unknown;
  };
  /**
   * Update the execution status.
   */
  status?: ToolCallStatus | null;
  /**
   * Update the human-readable title.
   */
  title?: string | null;
  /**
   * The ID of the tool call being updated.
   */
  toolCallId: string;
}
/**
 * Optional annotations for the client. The client can use annotations to inform how objects are used or displayed
 */
export interface Annotations {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  audience?: Role[] | null;
  lastModified?: string | null;
  priority?: number | null;
}
/**
 * Text-based resource contents.
 */
export interface TextResourceContents {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  mimeType?: string | null;
  text: string;
  uri: string;
}
/**
 * Binary resource contents.
 */
export interface BlobResourceContents {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  blob: string;
  mimeType?: string | null;
  uri: string;
}
/**
 * A file location being accessed or modified by a tool.
 *
 * Enables clients to implement "follow-along" features that track
 * which files the agent is working with in real-time.
 *
 * See protocol docs: [Following the Agent](https://agentclientprotocol.com/protocol/tool-calls#following-the-agent)
 */
export interface ToolCallLocation {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Optional line number within the file.
   */
  line?: number | null;
  /**
   * The file path being accessed or modified.
   */
  path: string;
}
export interface CreateTerminalRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  args?: string[];
  command: string;
  cwd?: string | null;
  env?: EnvVariable[];
  outputByteLimit?: number | null;
  sessionId: SessionId;
}
/**
 * An environment variable to set when launching an MCP server.
 */
export interface EnvVariable {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * The name of the environment variable.
   */
  name: string;
  /**
   * The value to set for the environment variable.
   */
  value: string;
}
export interface TerminalOutputRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  sessionId: SessionId;
  terminalId: string;
}
export interface ReleaseTerminalRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  sessionId: SessionId;
  terminalId: string;
}
export interface WaitForTerminalExitRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  sessionId: SessionId;
  terminalId: string;
}
export interface KillTerminalRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  sessionId: SessionId;
  terminalId: string;
}
/**
 * Request parameters for extension method calls.
 *
 * Used with the `_method` extension point to add custom request-response functionality
 * to the protocol while maintaining compatibility.
 *
 * See protocol docs: [Extension Methods](https://agentclientprotocol.com/protocol/extensibility#extension-methods)
 */
export interface ExtMethodRequest {
  /**
   * The identifier for the extension method.
   *
   * To help avoid conflicts, it's a good practice to prefix extension
   * methods with a unique identifier such as domain name.
   */
  method: string;
  /**
   * The parameters for the extension method, can be any JSON value.
   */
  params: {
    [k: string]: unknown;
  };
}
/**
 * Response containing the contents of a text file.
 */
export interface ReadTextFileResponse {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  content: string;
}
/**
 * Response to a permission request.
 */
export interface RequestPermissionResponse {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * The user's decision on the permission request.
   */
  outcome:
    | {
        outcome: "cancelled";
      }
    | {
        /**
         * The ID of the option the user selected.
         */
        optionId: string;
        outcome: "selected";
      };
}
export interface CreateTerminalResponse {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  terminalId: string;
}
export interface TerminalOutputResponse {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  exitStatus?: TerminalExitStatus | null;
  output: string;
  truncated: boolean;
}
export interface TerminalExitStatus {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  exitCode?: number | null;
  signal?: string | null;
}
export interface WaitForTerminalExitResponse {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  exitCode?: number | null;
  signal?: string | null;
}
/**
 * Response from extension method calls.
 *
 * Contains the result of a custom extension method request.
 *
 * See protocol docs: [Extension Methods](https://agentclientprotocol.com/protocol/extensibility#extension-methods)
 */
export interface ExtMethodResponse {
  [k: string]: unknown;
}
/**
 * Notification to cancel ongoing operations for a session.
 *
 * See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
 */
export interface CancelNotification {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * A unique identifier for a conversation session between a client and agent.
   *
   * Sessions maintain their own context, conversation history, and state,
   * allowing multiple independent interactions with the same agent.
   *
   * # Example
   *
   * ```
   * use agent_client_protocol::SessionId;
   * use std::sync::Arc;
   *
   * let session_id = SessionId(Arc::from("sess_abc123def456"));
   * ```
   *
   * See protocol docs: [Session ID](https://agentclientprotocol.com/protocol/session-setup#session-id)
   */
  sessionId: string;
}
/**
 * Extension notification parameters.
 *
 * Used with the `_notification` extension point to add custom one-way messages
 * to the protocol while maintaining compatibility.
 *
 * See protocol docs: [Extension Methods](https://agentclientprotocol.com/protocol/extensibility#extension-methods)
 */
export interface ExtNotification {
  /**
   * The identifier for the extension method.
   *
   * To help avoid conflicts, it's a good practice to prefix extension
   * methods with a unique identifier such as domain name.
   */
  method: string;
  /**
   * The parameters for the extension notification, can be any JSON value.
   */
  params: {
    [k: string]: unknown;
  };
}
/**
 * Request parameters for the initialize method.
 *
 * Sent by the client to establish connection and negotiate capabilities.
 *
 * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
 */
export interface InitializeRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  clientCapabilities?: ClientCapabilities;
  /**
   * The latest protocol version supported by the client.
   */
  protocolVersion: number;
}
/**
 * Capabilities supported by the client.
 */
export interface ClientCapabilities {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  fs?: FileSystemCapability;
  /**
   * **UNSTABLE**
   *
   * This capability is not part of the spec yet, and may be removed or changed at any point.
   */
  terminal?: boolean;
}
/**
 * File system capabilities supported by the client.
 * Determines which file operations the agent can request.
 */
export interface FileSystemCapability {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Whether the Client supports `fs/read_text_file` requests.
   */
  readTextFile?: boolean;
  /**
   * Whether the Client supports `fs/write_text_file` requests.
   */
  writeTextFile?: boolean;
}
/**
 * Request parameters for the authenticate method.
 *
 * Specifies which authentication method to use.
 */
export interface AuthenticateRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * The ID of the authentication method to use.
   * Must be one of the methods advertised in the initialize response.
   */
  methodId: string;
}
/**
 * Request parameters for creating a new session.
 *
 * See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 */
export interface NewSessionRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * The working directory for this session. Must be an absolute path.
   */
  cwd: string;
  /**
   * List of MCP (Model Context Protocol) servers the agent should connect to.
   */
  mcpServers: McpServer[];
}
/**
 * An HTTP header to set when making requests to the MCP server.
 */
export interface HttpHeader {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * The name of the HTTP header.
   */
  name: string;
  /**
   * The value to set for the HTTP header.
   */
  value: string;
}
/**
 * Stdio transport configuration
 *
 * All Agents MUST support this transport.
 */
export interface Stdio {
  /**
   * Command-line arguments to pass to the MCP server.
   */
  args: string[];
  /**
   * Path to the MCP server executable.
   */
  command: string;
  /**
   * Environment variables to set when launching the MCP server.
   */
  env: EnvVariable[];
  /**
   * Human-readable name identifying this MCP server.
   */
  name: string;
}
/**
 * Request parameters for loading an existing session.
 *
 * Only available if the Agent supports the `loadSession` capability.
 *
 * See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
 */
export interface LoadSessionRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * The working directory for this session.
   */
  cwd: string;
  /**
   * List of MCP servers to connect to for this session.
   */
  mcpServers: McpServer[];
  /**
   * A unique identifier for a conversation session between a client and agent.
   *
   * Sessions maintain their own context, conversation history, and state,
   * allowing multiple independent interactions with the same agent.
   *
   * # Example
   *
   * ```
   * use agent_client_protocol::SessionId;
   * use std::sync::Arc;
   *
   * let session_id = SessionId(Arc::from("sess_abc123def456"));
   * ```
   *
   * See protocol docs: [Session ID](https://agentclientprotocol.com/protocol/session-setup#session-id)
   */
  sessionId: string;
}
/**
 * **UNSTABLE**
 *
 * This type is not part of the spec, and may be removed or changed at any point.
 */
export interface SetSessionModeRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  modeId: SessionModeId;
  sessionId: SessionId;
}
/**
 * Request parameters for sending a user prompt to the agent.
 *
 * Contains the user's message and any additional context.
 *
 * See protocol docs: [User Message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
 */
export interface PromptRequest {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * The blocks of content that compose the user's message.
   *
   * As a baseline, the Agent MUST support [`ContentBlock::Text`] and [`ContentBlock::ResourceLink`],
   * while other variants are optionally enabled via [`PromptCapabilities`].
   *
   * The Client MUST adapt its interface according to [`PromptCapabilities`].
   *
   * The client MAY include referenced pieces of context as either
   * [`ContentBlock::Resource`] or [`ContentBlock::ResourceLink`].
   *
   * When available, [`ContentBlock::Resource`] is preferred
   * as it avoids extra round-trips and allows the message to include
   * pieces of context from sources the agent may not have access to.
   */
  prompt: ContentBlock[];
  /**
   * A unique identifier for a conversation session between a client and agent.
   *
   * Sessions maintain their own context, conversation history, and state,
   * allowing multiple independent interactions with the same agent.
   *
   * # Example
   *
   * ```
   * use agent_client_protocol::SessionId;
   * use std::sync::Arc;
   *
   * let session_id = SessionId(Arc::from("sess_abc123def456"));
   * ```
   *
   * See protocol docs: [Session ID](https://agentclientprotocol.com/protocol/session-setup#session-id)
   */
  sessionId: string;
}
/**
 * Request parameters for extension method calls.
 *
 * Used with the `_method` extension point to add custom request-response functionality
 * to the protocol while maintaining compatibility.
 *
 * See protocol docs: [Extension Methods](https://agentclientprotocol.com/protocol/extensibility#extension-methods)
 */
export interface ExtMethodRequest1 {
  /**
   * The identifier for the extension method.
   *
   * To help avoid conflicts, it's a good practice to prefix extension
   * methods with a unique identifier such as domain name.
   */
  method: string;
  /**
   * The parameters for the extension method, can be any JSON value.
   */
  params: {
    [k: string]: unknown;
  };
}
/**
 * Response from the initialize method.
 *
 * Contains the negotiated protocol version and agent capabilities.
 *
 * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
 */
export interface InitializeResponse {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  agentCapabilities?: AgentCapabilities;
  /**
   * Authentication methods supported by the agent.
   */
  authMethods?: AuthMethod[];
  /**
   * The protocol version the client specified if supported by the agent,
   * or the latest protocol version supported by the agent.
   *
   * The client should disconnect, if it doesn't support this version.
   */
  protocolVersion: number;
}
/**
 * Capabilities supported by the agent.
 */
export interface AgentCapabilities {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Whether the agent supports `session/load`.
   */
  loadSession?: boolean;
  mcpCapabilities?: McpCapabilities;
  promptCapabilities?: PromptCapabilities;
}
/**
 * MCP capabilities supported by the agent.
 */
export interface McpCapabilities {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Agent supports [`McpServer::Http`].
   */
  http?: boolean;
  /**
   * Agent supports [`McpServer::Sse`].
   */
  sse?: boolean;
}
/**
 * Prompt capabilities supported by the agent.
 */
export interface PromptCapabilities {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Agent supports [`ContentBlock::Audio`].
   */
  audio?: boolean;
  /**
   * Agent supports embedded context in `session/prompt` requests.
   *
   * When enabled, the Client is allowed to include [`ContentBlock::Resource`]
   * in prompt requests for pieces of context that are referenced in the message.
   */
  embeddedContext?: boolean;
  /**
   * Agent supports [`ContentBlock::Image`].
   */
  image?: boolean;
}
/**
 * Describes an available authentication method.
 */
export interface AuthMethod {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Optional description providing more details about this authentication method.
   */
  description?: string | null;
  /**
   * Unique identifier for this authentication method.
   */
  id: string;
  /**
   * Human-readable name of the authentication method.
   */
  name: string;
}
/**
 * Response from creating a new session.
 *
 * See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 */
export interface NewSessionResponse {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * **UNSTABLE**
   *
   * This field is not part of the spec, and may be removed or changed at any point.
   */
  modes?: SessionModeState | null;
  /**
   * A unique identifier for a conversation session between a client and agent.
   *
   * Sessions maintain their own context, conversation history, and state,
   * allowing multiple independent interactions with the same agent.
   *
   * # Example
   *
   * ```
   * use agent_client_protocol::SessionId;
   * use std::sync::Arc;
   *
   * let session_id = SessionId(Arc::from("sess_abc123def456"));
   * ```
   *
   * See protocol docs: [Session ID](https://agentclientprotocol.com/protocol/session-setup#session-id)
   */
  sessionId: string;
}
/**
 * **UNSTABLE**
 *
 * This type is not part of the spec, and may be removed or changed at any point.
 */
export interface SessionModeState {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  availableModes: SessionMode[];
  currentModeId: SessionModeId;
}
/**
 * **UNSTABLE**
 *
 * This type is not part of the spec, and may be removed or changed at any point.
 */
export interface SessionMode {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  description?: string | null;
  id: SessionModeId;
  name: string;
}
/**
 * Response from loading an existing session.
 */
export interface LoadSessionResponse {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * **UNSTABLE**
   *
   * This field is not part of the spec, and may be removed or changed at any point.
   */
  modes?: SessionModeState | null;
}
/**
 * **UNSTABLE**
 *
 * This type is not part of the spec, and may be removed or changed at any point.
 */
export interface SetSessionModeResponse {}
/**
 * Response from processing a user prompt.
 *
 * See protocol docs: [Check for Completion](https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion)
 */
export interface PromptResponse {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Indicates why the agent stopped processing the turn.
   */
  stopReason:
    | "end_turn"
    | "max_tokens"
    | "max_turn_requests"
    | "refusal"
    | "cancelled";
}
/**
 * Response from extension method calls.
 *
 * Contains the result of a custom extension method request.
 *
 * See protocol docs: [Extension Methods](https://agentclientprotocol.com/protocol/extensibility#extension-methods)
 */
export interface ExtMethodResponse1 {
  [k: string]: unknown;
}
/**
 * Notification containing a session update from the agent.
 *
 * Used to stream real-time progress and results during prompt processing.
 *
 * See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
 */
export interface SessionNotification {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * The ID of the session this update pertains to.
   */
  sessionId: string;
  /**
   * The actual update content.
   */
  update:
    | {
        content: ContentBlock;
        sessionUpdate: "user_message_chunk";
      }
    | {
        content: ContentBlock;
        sessionUpdate: "agent_message_chunk";
      }
    | {
        content: ContentBlock;
        sessionUpdate: "agent_thought_chunk";
      }
    | {
        /**
         * Extension point for implementations
         */
        _meta?: {
          [k: string]: unknown;
        };
        /**
         * Content produced by the tool call.
         */
        content?: ToolCallContent[];
        /**
         * The category of tool being invoked.
         * Helps clients choose appropriate icons and UI treatment.
         */
        kind?:
          | "read"
          | "edit"
          | "delete"
          | "move"
          | "search"
          | "execute"
          | "think"
          | "fetch"
          | "switch_mode"
          | "other";
        /**
         * File locations affected by this tool call.
         * Enables "follow-along" features in clients.
         */
        locations?: ToolCallLocation[];
        /**
         * Raw input parameters sent to the tool.
         */
        rawInput?: {
          [k: string]: unknown;
        };
        /**
         * Raw output returned by the tool.
         */
        rawOutput?: {
          [k: string]: unknown;
        };
        sessionUpdate: "tool_call";
        /**
         * Current execution status of the tool call.
         */
        status?: "pending" | "in_progress" | "completed" | "failed";
        /**
         * Human-readable title describing what the tool is doing.
         */
        title: string;
        /**
         * Unique identifier for this tool call within the session.
         */
        toolCallId: string;
      }
    | {
        /**
         * Extension point for implementations
         */
        _meta?: {
          [k: string]: unknown;
        };
        /**
         * Replace the content collection.
         */
        content?: ToolCallContent[] | null;
        /**
         * Update the tool kind.
         */
        kind?: ToolKind | null;
        /**
         * Replace the locations collection.
         */
        locations?: ToolCallLocation[] | null;
        /**
         * Update the raw input.
         */
        rawInput?: {
          [k: string]: unknown;
        };
        /**
         * Update the raw output.
         */
        rawOutput?: {
          [k: string]: unknown;
        };
        sessionUpdate: "tool_call_update";
        /**
         * Update the execution status.
         */
        status?: ToolCallStatus | null;
        /**
         * Update the human-readable title.
         */
        title?: string | null;
        /**
         * The ID of the tool call being updated.
         */
        toolCallId: string;
      }
    | {
        /**
         * Extension point for implementations
         */
        _meta?: {
          [k: string]: unknown;
        };
        /**
         * The list of tasks to be accomplished.
         *
         * When updating a plan, the agent must send a complete list of all entries
         * with their current status. The client replaces the entire plan with each update.
         */
        entries: PlanEntry[];
        sessionUpdate: "plan";
      }
    | {
        availableCommands: AvailableCommand[];
        sessionUpdate: "available_commands_update";
      }
    | {
        currentModeId: SessionModeId;
        sessionUpdate: "current_mode_update";
      };
}
/**
 * A single entry in the execution plan.
 *
 * Represents a task or goal that the assistant intends to accomplish
 * as part of fulfilling the user's request.
 * See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
 */
export interface PlanEntry {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Human-readable description of what this task aims to accomplish.
   */
  content: string;
  /**
   * The relative importance of this task.
   * Used to indicate which tasks are most critical to the overall goal.
   */
  priority: "high" | "medium" | "low";
  /**
   * Current execution status of this task.
   */
  status: "pending" | "in_progress" | "completed";
}
/**
 * Information about a command.
 */
export interface AvailableCommand {
  /**
   * Extension point for implementations
   */
  _meta?: {
    [k: string]: unknown;
  };
  /**
   * Human-readable description of what the command does.
   */
  description: string;
  /**
   * Input for the command if required
   */
  input?: AvailableCommandInput | null;
  /**
   * Command name (e.g., "create_plan", "research_codebase").
   */
  name: string;
}
/**
 * All text that was typed after the command name is provided as input.
 */
export interface UnstructuredCommandInput {
  /**
   * A brief description of the expected input
   */
  hint: string;
}
/**
 * Extension notification parameters.
 *
 * Used with the `_notification` extension point to add custom one-way messages
 * to the protocol while maintaining compatibility.
 *
 * See protocol docs: [Extension Methods](https://agentclientprotocol.com/protocol/extensibility#extension-methods)
 */
export interface ExtNotification1 {
  /**
   * The identifier for the extension method.
   *
   * To help avoid conflicts, it's a good practice to prefix extension
   * methods with a unique identifier such as domain name.
   */
  method: string;
  /**
   * The parameters for the extension notification, can be any JSON value.
   */
  params: {
    [k: string]: unknown;
  };
}

/** @internal */
export const writeTextFileRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  content: z.string(),
  path: z.string(),
  sessionId: z.string(),
});

/** @internal */
export const readTextFileRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  limit: z.number().optional().nullable(),
  line: z.number().optional().nullable(),
  path: z.string(),
  sessionId: z.string(),
});

/** @internal */
export const extMethodRequestSchema = z.object({
  method: z.string(),
  params: z.record(z.unknown()),
});

/** @internal */
export const roleSchema = z.union([z.literal("assistant"), z.literal("user")]);

/** @internal */
export const textResourceContentsSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  mimeType: z.string().optional().nullable(),
  text: z.string(),
  uri: z.string(),
});

/** @internal */
export const blobResourceContentsSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  blob: z.string(),
  mimeType: z.string().optional().nullable(),
  uri: z.string(),
});

/** @internal */
export const toolKindSchema = z.union([
  z.literal("read"),
  z.literal("edit"),
  z.literal("delete"),
  z.literal("move"),
  z.literal("search"),
  z.literal("execute"),
  z.literal("think"),
  z.literal("fetch"),
  z.literal("switch_mode"),
  z.literal("other"),
]);

/** @internal */
export const toolCallStatusSchema = z.union([
  z.literal("pending"),
  z.literal("in_progress"),
  z.literal("completed"),
  z.literal("failed"),
]);

/** @internal */
export const sessionIdSchema = z.string();

/** @internal */
export const writeTextFileResponseSchema = z.null();

/** @internal */
export const readTextFileResponseSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  content: z.string(),
});

/** @internal */
export const requestPermissionResponseSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  outcome: z.union([
    z.object({
      outcome: z.literal("cancelled"),
    }),
    z.object({
      optionId: z.string(),
      outcome: z.literal("selected"),
    }),
  ]),
});

/** @internal */
export const createTerminalResponseSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  terminalId: z.string(),
});

/** @internal */
export const releaseTerminalResponseSchema = z.null();

/** @internal */
export const waitForTerminalExitResponseSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  exitCode: z.number().optional().nullable(),
  signal: z.string().optional().nullable(),
});

/** @internal */
export const killTerminalResponseSchema = z.null();

/** @internal */
export const extMethodResponseSchema = z.record(z.unknown());

/** @internal */
export const cancelNotificationSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  sessionId: z.string(),
});

/** @internal */
export const extNotificationSchema = z.object({
  method: z.string(),
  params: z.record(z.unknown()),
});

/** @internal */
export const authenticateRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  methodId: z.string(),
});

/** @internal */
export const extMethodRequest1Schema = z.object({
  method: z.string(),
  params: z.record(z.unknown()),
});

/** @internal */
export const httpHeaderSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  name: z.string(),
  value: z.string(),
});

/** @internal */
export const sessionModeIdSchema = z.string();

/** @internal */
export const annotationsSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  audience: z.array(roleSchema).optional().nullable(),
  lastModified: z.string().optional().nullable(),
  priority: z.number().optional().nullable(),
});

/** @internal */
export const embeddedResourceResourceSchema = z.union([
  textResourceContentsSchema,
  blobResourceContentsSchema,
]);

/** @internal */
export const authenticateResponseSchema = z.null();

/** @internal */
export const setSessionModeResponseSchema = z.object({});

/** @internal */
export const promptResponseSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  stopReason: z.union([
    z.literal("end_turn"),
    z.literal("max_tokens"),
    z.literal("max_turn_requests"),
    z.literal("refusal"),
    z.literal("cancelled"),
  ]),
});

/** @internal */
export const extMethodResponse1Schema = z.record(z.unknown());

/** @internal */
export const extNotification1Schema = z.object({
  method: z.string(),
  params: z.record(z.unknown()),
});

/** @internal */
export const unstructuredCommandInputSchema = z.object({
  hint: z.string(),
});

/** @internal */
export const permissionOptionSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  kind: z.union([
    z.literal("allow_once"),
    z.literal("allow_always"),
    z.literal("reject_once"),
    z.literal("reject_always"),
  ]),
  name: z.string(),
  optionId: z.string(),
});

/** @internal */
export const toolCallContentSchema = z.union([
  z.object({
    content: z.union([
      z.object({
        _meta: z.record(z.unknown()).optional(),
        annotations: annotationsSchema.optional().nullable(),
        text: z.string(),
        type: z.literal("text"),
      }),
      z.object({
        _meta: z.record(z.unknown()).optional(),
        annotations: annotationsSchema.optional().nullable(),
        data: z.string(),
        mimeType: z.string(),
        type: z.literal("image"),
        uri: z.string().optional().nullable(),
      }),
      z.object({
        _meta: z.record(z.unknown()).optional(),
        annotations: annotationsSchema.optional().nullable(),
        data: z.string(),
        mimeType: z.string(),
        type: z.literal("audio"),
      }),
      z.object({
        _meta: z.record(z.unknown()).optional(),
        annotations: annotationsSchema.optional().nullable(),
        description: z.string().optional().nullable(),
        mimeType: z.string().optional().nullable(),
        name: z.string(),
        size: z.number().optional().nullable(),
        title: z.string().optional().nullable(),
        type: z.literal("resource_link"),
        uri: z.string(),
      }),
      z.object({
        _meta: z.record(z.unknown()).optional(),
        annotations: annotationsSchema.optional().nullable(),
        resource: embeddedResourceResourceSchema,
        type: z.literal("resource"),
      }),
    ]),
    type: z.literal("content"),
  }),
  z.object({
    _meta: z.record(z.unknown()).optional(),
    newText: z.string(),
    oldText: z.string().optional().nullable(),
    path: z.string(),
    type: z.literal("diff"),
  }),
  z.object({
    terminalId: z.string(),
    type: z.literal("terminal"),
  }),
]);

/** @internal */
export const toolCallLocationSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  line: z.number().optional().nullable(),
  path: z.string(),
});

/** @internal */
export const envVariableSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  name: z.string(),
  value: z.string(),
});

/** @internal */
export const terminalOutputRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  sessionId: sessionIdSchema,
  terminalId: z.string(),
});

/** @internal */
export const releaseTerminalRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  sessionId: sessionIdSchema,
  terminalId: z.string(),
});

/** @internal */
export const waitForTerminalExitRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  sessionId: sessionIdSchema,
  terminalId: z.string(),
});

/** @internal */
export const killTerminalRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  sessionId: sessionIdSchema,
  terminalId: z.string(),
});

/** @internal */
export const terminalExitStatusSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  exitCode: z.number().optional().nullable(),
  signal: z.string().optional().nullable(),
});

/** @internal */
export const fileSystemCapabilitySchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  readTextFile: z.boolean().optional(),
  writeTextFile: z.boolean().optional(),
});

/** @internal */
export const stdioSchema = z.object({
  args: z.array(z.string()),
  command: z.string(),
  env: z.array(envVariableSchema),
  name: z.string(),
});

/** @internal */
export const mcpServerSchema = z.union([
  z.object({
    headers: z.array(httpHeaderSchema),
    name: z.string(),
    type: z.literal("http"),
    url: z.string(),
  }),
  z.object({
    headers: z.array(httpHeaderSchema),
    name: z.string(),
    type: z.literal("sse"),
    url: z.string(),
  }),
  stdioSchema,
]);

/** @internal */
export const setSessionModeRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  modeId: sessionModeIdSchema,
  sessionId: sessionIdSchema,
});

/** @internal */
export const contentBlockSchema = z.union([
  z.object({
    _meta: z.record(z.unknown()).optional(),
    annotations: annotationsSchema.optional().nullable(),
    text: z.string(),
    type: z.literal("text"),
  }),
  z.object({
    _meta: z.record(z.unknown()).optional(),
    annotations: annotationsSchema.optional().nullable(),
    data: z.string(),
    mimeType: z.string(),
    type: z.literal("image"),
    uri: z.string().optional().nullable(),
  }),
  z.object({
    _meta: z.record(z.unknown()).optional(),
    annotations: annotationsSchema.optional().nullable(),
    data: z.string(),
    mimeType: z.string(),
    type: z.literal("audio"),
  }),
  z.object({
    _meta: z.record(z.unknown()).optional(),
    annotations: annotationsSchema.optional().nullable(),
    description: z.string().optional().nullable(),
    mimeType: z.string().optional().nullable(),
    name: z.string(),
    size: z.number().optional().nullable(),
    title: z.string().optional().nullable(),
    type: z.literal("resource_link"),
    uri: z.string(),
  }),
  z.object({
    _meta: z.record(z.unknown()).optional(),
    annotations: annotationsSchema.optional().nullable(),
    resource: embeddedResourceResourceSchema,
    type: z.literal("resource"),
  }),
]);

/** @internal */
export const authMethodSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  description: z.string().optional().nullable(),
  id: z.string(),
  name: z.string(),
});

/** @internal */
export const mcpCapabilitiesSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  http: z.boolean().optional(),
  sse: z.boolean().optional(),
});

/** @internal */
export const promptCapabilitiesSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  audio: z.boolean().optional(),
  embeddedContext: z.boolean().optional(),
  image: z.boolean().optional(),
});

/** @internal */
export const sessionModeSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  description: z.string().optional().nullable(),
  id: sessionModeIdSchema,
  name: z.string(),
});

/** @internal */
export const sessionModeStateSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  availableModes: z.array(sessionModeSchema),
  currentModeId: sessionModeIdSchema,
});

/** @internal */
export const planEntrySchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  content: z.string(),
  priority: z.union([z.literal("high"), z.literal("medium"), z.literal("low")]),
  status: z.union([
    z.literal("pending"),
    z.literal("in_progress"),
    z.literal("completed"),
  ]),
});

/** @internal */
export const availableCommandInputSchema = unstructuredCommandInputSchema;

/** @internal */
export const clientNotificationSchema = z.union([
  cancelNotificationSchema,
  extNotificationSchema,
]);

/** @internal */
export const createTerminalRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  args: z.array(z.string()).optional(),
  command: z.string(),
  cwd: z.string().optional().nullable(),
  env: z.array(envVariableSchema).optional(),
  outputByteLimit: z.number().optional().nullable(),
  sessionId: sessionIdSchema,
});

/** @internal */
export const terminalOutputResponseSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  exitStatus: terminalExitStatusSchema.optional().nullable(),
  output: z.string(),
  truncated: z.boolean(),
});

/** @internal */
export const newSessionRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  cwd: z.string(),
  mcpServers: z.array(mcpServerSchema),
});

/** @internal */
export const loadSessionRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  cwd: z.string(),
  mcpServers: z.array(mcpServerSchema),
  sessionId: z.string(),
});

/** @internal */
export const promptRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  prompt: z.array(contentBlockSchema),
  sessionId: z.string(),
});

/** @internal */
export const newSessionResponseSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  modes: sessionModeStateSchema.optional().nullable(),
  sessionId: z.string(),
});

/** @internal */
export const loadSessionResponseSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  modes: sessionModeStateSchema.optional().nullable(),
});

/** @internal */
export const toolCallUpdateSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  content: z.array(toolCallContentSchema).optional().nullable(),
  kind: toolKindSchema.optional().nullable(),
  locations: z.array(toolCallLocationSchema).optional().nullable(),
  rawInput: z.record(z.unknown()).optional(),
  rawOutput: z.record(z.unknown()).optional(),
  status: toolCallStatusSchema.optional().nullable(),
  title: z.string().optional().nullable(),
  toolCallId: z.string(),
});

/** @internal */
export const clientCapabilitiesSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  fs: fileSystemCapabilitySchema.optional(),
  terminal: z.boolean().optional(),
});

/** @internal */
export const agentCapabilitiesSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  loadSession: z.boolean().optional(),
  mcpCapabilities: mcpCapabilitiesSchema.optional(),
  promptCapabilities: promptCapabilitiesSchema.optional(),
});

/** @internal */
export const availableCommandSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  description: z.string(),
  input: availableCommandInputSchema.optional().nullable(),
  name: z.string(),
});

/** @internal */
export const clientResponseSchema = z.union([
  writeTextFileResponseSchema,
  readTextFileResponseSchema,
  requestPermissionResponseSchema,
  createTerminalResponseSchema,
  terminalOutputResponseSchema,
  releaseTerminalResponseSchema,
  waitForTerminalExitResponseSchema,
  killTerminalResponseSchema,
  extMethodResponseSchema,
]);

/** @internal */
export const requestPermissionRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  options: z.array(permissionOptionSchema),
  sessionId: z.string(),
  toolCall: toolCallUpdateSchema,
});

/** @internal */
export const initializeRequestSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  clientCapabilities: clientCapabilitiesSchema.optional(),
  protocolVersion: z.number(),
});

/** @internal */
export const initializeResponseSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  agentCapabilities: agentCapabilitiesSchema.optional(),
  authMethods: z.array(authMethodSchema).optional(),
  protocolVersion: z.number(),
});

/** @internal */
export const sessionNotificationSchema = z.object({
  _meta: z.record(z.unknown()).optional(),
  sessionId: z.string(),
  update: z.union([
    z.object({
      content: contentBlockSchema,
      sessionUpdate: z.literal("user_message_chunk"),
    }),
    z.object({
      content: contentBlockSchema,
      sessionUpdate: z.literal("agent_message_chunk"),
    }),
    z.object({
      content: contentBlockSchema,
      sessionUpdate: z.literal("agent_thought_chunk"),
    }),
    z.object({
      _meta: z.record(z.unknown()).optional(),
      content: z.array(toolCallContentSchema).optional(),
      kind: z
        .union([
          z.literal("read"),
          z.literal("edit"),
          z.literal("delete"),
          z.literal("move"),
          z.literal("search"),
          z.literal("execute"),
          z.literal("think"),
          z.literal("fetch"),
          z.literal("switch_mode"),
          z.literal("other"),
        ])
        .optional(),
      locations: z.array(toolCallLocationSchema).optional(),
      rawInput: z.record(z.unknown()).optional(),
      rawOutput: z.record(z.unknown()).optional(),
      sessionUpdate: z.literal("tool_call"),
      status: z
        .union([
          z.literal("pending"),
          z.literal("in_progress"),
          z.literal("completed"),
          z.literal("failed"),
        ])
        .optional(),
      title: z.string(),
      toolCallId: z.string(),
    }),
    z.object({
      _meta: z.record(z.unknown()).optional(),
      content: z.array(toolCallContentSchema).optional().nullable(),
      kind: toolKindSchema.optional().nullable(),
      locations: z.array(toolCallLocationSchema).optional().nullable(),
      rawInput: z.record(z.unknown()).optional(),
      rawOutput: z.record(z.unknown()).optional(),
      sessionUpdate: z.literal("tool_call_update"),
      status: toolCallStatusSchema.optional().nullable(),
      title: z.string().optional().nullable(),
      toolCallId: z.string(),
    }),
    z.object({
      _meta: z.record(z.unknown()).optional(),
      entries: z.array(planEntrySchema),
      sessionUpdate: z.literal("plan"),
    }),
    z.object({
      availableCommands: z.array(availableCommandSchema),
      sessionUpdate: z.literal("available_commands_update"),
    }),
    z.object({
      currentModeId: sessionModeIdSchema,
      sessionUpdate: z.literal("current_mode_update"),
    }),
  ]),
});

/** @internal */
export const clientRequestSchema = z.union([
  writeTextFileRequestSchema,
  readTextFileRequestSchema,
  requestPermissionRequestSchema,
  createTerminalRequestSchema,
  terminalOutputRequestSchema,
  releaseTerminalRequestSchema,
  waitForTerminalExitRequestSchema,
  killTerminalRequestSchema,
  extMethodRequestSchema,
]);

/** @internal */
export const agentRequestSchema = z.union([
  initializeRequestSchema,
  authenticateRequestSchema,
  newSessionRequestSchema,
  loadSessionRequestSchema,
  setSessionModeRequestSchema,
  promptRequestSchema,
  extMethodRequest1Schema,
]);

/** @internal */
export const agentResponseSchema = z.union([
  initializeResponseSchema,
  authenticateResponseSchema,
  newSessionResponseSchema,
  loadSessionResponseSchema,
  setSessionModeResponseSchema,
  promptResponseSchema,
  extMethodResponse1Schema,
]);

/** @internal */
export const agentNotificationSchema = z.union([
  sessionNotificationSchema,
  extNotification1Schema,
]);

/** @internal */
export const agentClientProtocolSchema = z.union([
  clientRequestSchema,
  clientResponseSchema,
  clientNotificationSchema,
  agentRequestSchema,
  agentResponseSchema,
  agentNotificationSchema,
]);
