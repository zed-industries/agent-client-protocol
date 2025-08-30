export const AGENT_METHODS = {
  authenticate: "authenticate",
  initialize: "initialize",
  session_cancel: "session/cancel",
  session_list_commands: "session/list_commands",
  session_load: "session/load",
  session_new: "session/new",
  session_prompt: "session/prompt",
  session_run_command: "session/run_command",
};

export const CLIENT_METHODS = {
  fs_read_text_file: "fs/read_text_file",
  fs_write_text_file: "fs/write_text_file",
  session_request_permission: "session/request_permission",
  session_update: "session/update",
  terminal_create: "terminal/create",
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
  | WaitForTerminalExitRequest;
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
            annotations?: Annotations | null;
            text: string;
            type: "text";
          }
        | {
            annotations?: Annotations | null;
            data: string;
            mimeType: string;
            type: "image";
            uri?: string | null;
          }
        | {
            annotations?: Annotations | null;
            data: string;
            mimeType: string;
            type: "audio";
          }
        | {
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
            annotations?: Annotations | null;
            resource: EmbeddedResourceResource;
            type: "resource";
          };
      type: "content";
    }
  | {
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
  | WaitForTerminalExitResponse;
export type WriteTextFileResponse = null;
export type ReleaseTerminalResponse = null;
/**
 * All possible notifications that a client can send to an agent.
 *
 * This enum is used internally for routing RPC notifications. You typically won't need
 * to use this directly - use the notification methods on the [`Agent`] trait instead.
 *
 * Notifications do not expect a response.
 */
/** @internal */
export type ClientNotification = CancelNotification;
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
  | PromptRequest
  | ListCommandsRequest
  | RunCommandRequest;
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
      annotations?: Annotations | null;
      text: string;
      type: "text";
    }
  | {
      annotations?: Annotations | null;
      data: string;
      mimeType: string;
      type: "image";
      uri?: string | null;
    }
  | {
      annotations?: Annotations | null;
      data: string;
      mimeType: string;
      type: "audio";
    }
  | {
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
  | PromptResponse
  | ListCommandsResponse;
export type AuthenticateResponse = null;
export type LoadSessionResponse = null;
/**
 * All possible notifications that an agent can send to a client.
 *
 * This enum is used internally for routing RPC notifications. You typically won't need
 * to use this directly - use the notification methods on the [`Client`] trait instead.
 *
 * Notifications do not expect a response.
 */
/** @internal */
export type AgentNotification = SessionNotification;

/**
 * Request to write content to a text file.
 *
 * Only available if the client supports the `fs.writeTextFile` capability.
 */
export interface WriteTextFileRequest {
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
  audience?: Role[] | null;
  lastModified?: string | null;
  priority?: number | null;
}
/**
 * Text-based resource contents.
 */
export interface TextResourceContents {
  mimeType?: string | null;
  text: string;
  uri: string;
}
/**
 * Binary resource contents.
 */
export interface BlobResourceContents {
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
   * Optional line number within the file.
   */
  line?: number | null;
  /**
   * The file path being accessed or modified.
   */
  path: string;
}
export interface CreateTerminalRequest {
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
   * The name of the environment variable.
   */
  name: string;
  /**
   * The value to set for the environment variable.
   */
  value: string;
}
export interface TerminalOutputRequest {
  sessionId: SessionId;
  terminalId: string;
}
export interface ReleaseTerminalRequest {
  sessionId: SessionId;
  terminalId: string;
}
export interface WaitForTerminalExitRequest {
  sessionId: SessionId;
  terminalId: string;
}
/**
 * Response containing the contents of a text file.
 */
export interface ReadTextFileResponse {
  content: string;
}
/**
 * Response to a permission request.
 */
export interface RequestPermissionResponse {
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
  terminalId: string;
}
export interface TerminalOutputResponse {
  exitStatus?: TerminalExitStatus | null;
  output: string;
  truncated: boolean;
}
export interface TerminalExitStatus {
  exitCode?: number | null;
  signal?: string | null;
}
export interface WaitForTerminalExitResponse {
  exitCode?: number | null;
  signal?: string | null;
}
/**
 * Notification to cancel ongoing operations for a session.
 *
 * See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
 */
export interface CancelNotification {
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
 * Request parameters for the initialize method.
 *
 * Sent by the client to establish connection and negotiate capabilities.
 *
 * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
 */
export interface InitializeRequest {
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
   * The working directory for this session. Must be an absolute path.
   */
  cwd: string;
  /**
   * List of MCP (Model Context Protocol) servers the agent should connect to.
   */
  mcpServers: McpServer[];
}
/**
 * Configuration for connecting to an MCP (Model Context Protocol) server.
 *
 * MCP servers provide tools and context that the agent can use when
 * processing prompts.
 *
 * See protocol docs: [MCP Servers](https://agentclientprotocol.com/protocol/session-setup#mcp-servers)
 */
export interface McpServer {
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
 * Only available if the agent supports the `loadSession` capability.
 *
 * See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
 */
export interface LoadSessionRequest {
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
 * Request parameters for sending a user prompt to the agent.
 *
 * Contains the user's message and any additional context.
 *
 * See protocol docs: [User Message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
 */
export interface PromptRequest {
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
 * Request parameters for listing available commands.
 */
export interface ListCommandsRequest {
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
 * Request parameters for executing a command.
 */
export interface RunCommandRequest {
  /**
   * Optional arguments for the command.
   */
  args?: string | null;
  /**
   * Name of the command to execute.
   */
  command: string;
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
 * Response from the initialize method.
 *
 * Contains the negotiated protocol version and agent capabilities.
 *
 * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
 */
export interface InitializeResponse {
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
   * Whether the agent supports `session/load`.
   */
  loadSession?: boolean;
  promptCapabilities?: PromptCapabilities;
  /**
   * Agent supports commands via `list_commands` and `run_command`.
   */
  supportsCommands?: boolean;
}
/**
 * Prompt capabilities supported by the agent.
 */
export interface PromptCapabilities {
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
 * Response from processing a user prompt.
 *
 * See protocol docs: [Check for Completion](https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion)
 */
export interface PromptResponse {
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
 * Response containing available commands.
 */
export interface ListCommandsResponse {
  /**
   * List of available commands.
   */
  commands: CommandInfo[];
}
/**
 * Information about a custom command.
 */
export interface CommandInfo {
  /**
   * Human-readable description of what the command does.
   */
  description: string;
  /**
   * Command name (e.g., "create_plan", "research_codebase").
   */
  name: string;
  /**
   * Whether this command requires arguments from the user.
   */
  requiresArgument: boolean;
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
         * The list of tasks to be accomplished.
         *
         * When updating a plan, the agent must send a complete list of all entries
         * with their current status. The client replaces the entire plan with each update.
         */
        entries: PlanEntry[];
        sessionUpdate: "plan";
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

/** @internal */
export const writeTextFileRequestSchema = z.object({
  content: z.string(),
  path: z.string(),
  sessionId: z.string(),
});

/** @internal */
export const readTextFileRequestSchema = z.object({
  limit: z.number().optional().nullable(),
  line: z.number().optional().nullable(),
  path: z.string(),
  sessionId: z.string(),
});

/** @internal */
export const roleSchema = z.union([z.literal("assistant"), z.literal("user")]);

/** @internal */
export const textResourceContentsSchema = z.object({
  mimeType: z.string().optional().nullable(),
  text: z.string(),
  uri: z.string(),
});

/** @internal */
export const blobResourceContentsSchema = z.object({
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
  content: z.string(),
});

/** @internal */
export const requestPermissionResponseSchema = z.object({
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
  terminalId: z.string(),
});

/** @internal */
export const releaseTerminalResponseSchema = z.null();

/** @internal */
export const waitForTerminalExitResponseSchema = z.object({
  exitCode: z.number().optional().nullable(),
  signal: z.string().optional().nullable(),
});

/** @internal */
export const cancelNotificationSchema = z.object({
  sessionId: z.string(),
});

/** @internal */
export const authenticateRequestSchema = z.object({
  methodId: z.string(),
});

/** @internal */
export const listCommandsRequestSchema = z.object({
  sessionId: z.string(),
});

/** @internal */
export const runCommandRequestSchema = z.object({
  args: z.string().optional().nullable(),
  command: z.string(),
  sessionId: z.string(),
});

/** @internal */
export const annotationsSchema = z.object({
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
export const newSessionResponseSchema = z.object({
  sessionId: z.string(),
});

/** @internal */
export const loadSessionResponseSchema = z.null();

/** @internal */
export const promptResponseSchema = z.object({
  stopReason: z.union([
    z.literal("end_turn"),
    z.literal("max_tokens"),
    z.literal("max_turn_requests"),
    z.literal("refusal"),
    z.literal("cancelled"),
  ]),
});

/** @internal */
export const permissionOptionSchema = z.object({
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
        annotations: annotationsSchema.optional().nullable(),
        text: z.string(),
        type: z.literal("text"),
      }),
      z.object({
        annotations: annotationsSchema.optional().nullable(),
        data: z.string(),
        mimeType: z.string(),
        type: z.literal("image"),
        uri: z.string().optional().nullable(),
      }),
      z.object({
        annotations: annotationsSchema.optional().nullable(),
        data: z.string(),
        mimeType: z.string(),
        type: z.literal("audio"),
      }),
      z.object({
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
        annotations: annotationsSchema.optional().nullable(),
        resource: embeddedResourceResourceSchema,
        type: z.literal("resource"),
      }),
    ]),
    type: z.literal("content"),
  }),
  z.object({
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
  line: z.number().optional().nullable(),
  path: z.string(),
});

/** @internal */
export const envVariableSchema = z.object({
  name: z.string(),
  value: z.string(),
});

/** @internal */
export const terminalOutputRequestSchema = z.object({
  sessionId: sessionIdSchema,
  terminalId: z.string(),
});

/** @internal */
export const releaseTerminalRequestSchema = z.object({
  sessionId: sessionIdSchema,
  terminalId: z.string(),
});

/** @internal */
export const waitForTerminalExitRequestSchema = z.object({
  sessionId: sessionIdSchema,
  terminalId: z.string(),
});

/** @internal */
export const terminalExitStatusSchema = z.object({
  exitCode: z.number().optional().nullable(),
  signal: z.string().optional().nullable(),
});

/** @internal */
export const fileSystemCapabilitySchema = z.object({
  readTextFile: z.boolean().optional(),
  writeTextFile: z.boolean().optional(),
});

/** @internal */
export const mcpServerSchema = z.object({
  args: z.array(z.string()),
  command: z.string(),
  env: z.array(envVariableSchema),
  name: z.string(),
});

/** @internal */
export const loadSessionRequestSchema = z.object({
  cwd: z.string(),
  mcpServers: z.array(mcpServerSchema),
  sessionId: z.string(),
});

/** @internal */
export const contentBlockSchema = z.union([
  z.object({
    annotations: annotationsSchema.optional().nullable(),
    text: z.string(),
    type: z.literal("text"),
  }),
  z.object({
    annotations: annotationsSchema.optional().nullable(),
    data: z.string(),
    mimeType: z.string(),
    type: z.literal("image"),
    uri: z.string().optional().nullable(),
  }),
  z.object({
    annotations: annotationsSchema.optional().nullable(),
    data: z.string(),
    mimeType: z.string(),
    type: z.literal("audio"),
  }),
  z.object({
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
    annotations: annotationsSchema.optional().nullable(),
    resource: embeddedResourceResourceSchema,
    type: z.literal("resource"),
  }),
]);

/** @internal */
export const authMethodSchema = z.object({
  description: z.string().optional().nullable(),
  id: z.string(),
  name: z.string(),
});

/** @internal */
export const promptCapabilitiesSchema = z.object({
  audio: z.boolean().optional(),
  embeddedContext: z.boolean().optional(),
  image: z.boolean().optional(),
});

/** @internal */
export const commandInfoSchema = z.object({
  description: z.string(),
  name: z.string(),
  requiresArgument: z.boolean(),
});

/** @internal */
export const planEntrySchema = z.object({
  content: z.string(),
  priority: z.union([z.literal("high"), z.literal("medium"), z.literal("low")]),
  status: z.union([
    z.literal("pending"),
    z.literal("in_progress"),
    z.literal("completed"),
  ]),
});

/** @internal */
export const clientNotificationSchema = cancelNotificationSchema;

/** @internal */
export const createTerminalRequestSchema = z.object({
  args: z.array(z.string()).optional(),
  command: z.string(),
  cwd: z.string().optional().nullable(),
  env: z.array(envVariableSchema).optional(),
  outputByteLimit: z.number().optional().nullable(),
  sessionId: sessionIdSchema,
});

/** @internal */
export const terminalOutputResponseSchema = z.object({
  exitStatus: terminalExitStatusSchema.optional().nullable(),
  output: z.string(),
  truncated: z.boolean(),
});

/** @internal */
export const newSessionRequestSchema = z.object({
  cwd: z.string(),
  mcpServers: z.array(mcpServerSchema),
});

/** @internal */
export const promptRequestSchema = z.object({
  prompt: z.array(contentBlockSchema),
  sessionId: z.string(),
});

/** @internal */
export const listCommandsResponseSchema = z.object({
  commands: z.array(commandInfoSchema),
});

/** @internal */
export const sessionNotificationSchema = z.object({
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
      entries: z.array(planEntrySchema),
      sessionUpdate: z.literal("plan"),
    }),
  ]),
});

/** @internal */
export const toolCallUpdateSchema = z.object({
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
  fs: fileSystemCapabilitySchema.optional(),
  terminal: z.boolean().optional(),
});

/** @internal */
export const agentCapabilitiesSchema = z.object({
  loadSession: z.boolean().optional(),
  promptCapabilities: promptCapabilitiesSchema.optional(),
  supportsCommands: z.boolean().optional(),
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
]);

/** @internal */
export const agentNotificationSchema = sessionNotificationSchema;

/** @internal */
export const requestPermissionRequestSchema = z.object({
  options: z.array(permissionOptionSchema),
  sessionId: z.string(),
  toolCall: toolCallUpdateSchema,
});

/** @internal */
export const initializeRequestSchema = z.object({
  clientCapabilities: clientCapabilitiesSchema.optional(),
  protocolVersion: z.number(),
});

/** @internal */
export const initializeResponseSchema = z.object({
  agentCapabilities: agentCapabilitiesSchema.optional(),
  authMethods: z.array(authMethodSchema).optional(),
  protocolVersion: z.number(),
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
]);

/** @internal */
export const agentRequestSchema = z.union([
  initializeRequestSchema,
  authenticateRequestSchema,
  newSessionRequestSchema,
  loadSessionRequestSchema,
  promptRequestSchema,
  listCommandsRequestSchema,
  runCommandRequestSchema,
]);

/** @internal */
export const agentResponseSchema = z.union([
  initializeResponseSchema,
  authenticateResponseSchema,
  newSessionResponseSchema,
  loadSessionResponseSchema,
  promptResponseSchema,
  listCommandsResponseSchema,
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
