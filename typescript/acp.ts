export const AGENT_METHODS = {
  new_session: "acp/new_session",
  load_session: "acp/load_session",
  prompt: "acp/prompt",
  session_update: "acp/session_update",
};

export type AgentClientProtocol =
  | NewSessionArguments
  | NewSessionOutput
  | LoadSession
  | Prompt
  | SessionUpdate
  | RequestPermissionArguments
  | RequestPermissionOutput
  | WriteTextFile
  | ReadTextFileArguments
  | ReadTextFileOutput;
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
 * The sender or recipient of messages and data in a conversation.
 */
export type Role = "assistant" | "user";
export type EmbeddedResourceResource =
  | TextResourceContents
  | BlobResourceContents;
export type SessionUpdate =
  | {
      content: ContentBlock;
      sessionUpdate: "userMessageChunk";
    }
  | {
      content: ContentBlock;
      sessionUpdate: "agentMessageChunk";
    }
  | {
      content: ContentBlock;
      sessionUpdate: "agentThoughtChunk";
    }
  | {
      content?: ToolCallContent[];
      kind: ToolKind;
      label: string;
      locations?: ToolCallLocation[];
      rawInput?: unknown;
      sessionUpdate: "toolCall";
      status: ToolCallStatus;
      toolCallId: string;
    }
  | {
      content?: ToolCallContent[] | null;
      kind?: ToolKind | null;
      label?: string | null;
      locations?: ToolCallLocation[] | null;
      rawInput?: unknown;
      sessionUpdate: "toolCallUpdate";
      status?: ToolCallStatus | null;
      toolCallId: string;
    }
  | {
      entries: PlanEntry[];
      sessionUpdate: "plan";
    };
export type ToolCallContent =
  | {
      content: ContentBlock;
      type: "content";
    }
  | {
      newText: string;
      oldText: string | null;
      path: string;
      type: "diff";
    };
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
export type ToolCallStatus = "pending" | "inProgress" | "completed" | "failed";
export type PermissionOptionKind =
  | "allowOnce"
  | "allowAlways"
  | "rejectOnce"
  | "rejectAlways";
export type RequestPermissionOutcome =
  | {
      outcome: "canceled";
    }
  | {
      optionId: string;
      outcome: "selected";
    };

export interface NewSessionArguments {
  clientTools: ClientTools;
  cwd: string;
  mcpServers: {
    [k: string]: McpServerConfig;
  };
}
export interface ClientTools {
  readTextFile: McpToolId | null;
  requestPermission: McpToolId | null;
  writeTextFile: McpToolId | null;
}
export interface McpToolId {
  mcpServer: string;
  toolName: string;
}
export interface McpServerConfig {
  args: string[];
  command: string;
  env?: {
    [k: string]: string;
  } | null;
}
export interface NewSessionOutput {
  sessionId: string;
}
export interface LoadSession {
  clientTools: ClientTools;
  cwd: string;
  mcpServers: {
    [k: string]: McpServerConfig;
  };
  sessionId: string;
}
export interface Prompt {
  prompt: ContentBlock[];
  sessionId: string;
}
/**
 * Optional annotations for the client. The client can use annotations to inform how objects are used or displayed
 */
export interface Annotations {
  audience?: Role[] | null;
  lastModified?: string | null;
  priority?: number | null;
}
export interface TextResourceContents {
  mimeType?: string | null;
  text: string;
  uri: string;
}
export interface BlobResourceContents {
  blob: string;
  mimeType?: string | null;
  uri: string;
}
export interface ToolCallLocation {
  line?: number | null;
  path: string;
}
/**
 * A single entry in the execution plan.
 *
 * Represents a task or goal that the assistant intends to accomplish
 * as part of fulfilling the user's request.
 */
export interface PlanEntry {
  /**
   * Description of what this task aims to accomplish
   */
  content: string;
  /**
   * Relative importance of this task
   */
  priority: "high" | "medium" | "low";
  /**
   * Current progress of this task
   */
  status: "pending" | "in_progress" | "completed";
}
export interface RequestPermissionArguments {
  options: PermissionOption[];
  sessionId: string;
  toolCall: ToolCall;
}
export interface PermissionOption {
  kind: PermissionOptionKind;
  label: string;
  optionId: string;
}
export interface ToolCall {
  content?: ToolCallContent[];
  kind: ToolKind;
  label: string;
  locations?: ToolCallLocation[];
  rawInput?: unknown;
  status: ToolCallStatus;
  toolCallId: string;
}
export interface RequestPermissionOutput {
  outcome: RequestPermissionOutcome;
}
export interface WriteTextFile {
  content: string;
  path: string;
  sessionId: string;
}
export interface ReadTextFileArguments {
  limit?: number | null;
  line?: number | null;
  path: string;
  sessionId: string;
}
export interface ReadTextFileOutput {
  content: string;
}
