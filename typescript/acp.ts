export const NEW_SESSION_TOOL_NAME = "acp/new_session";
export const LOAD_SESSION_TOOL_NAME = "acp/load_session";
export const PROMPT_TOOL_NAME = "acp/prompt";

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
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource;
export type SessionUpdate =
  | ContentBlock1
  | ContentBlock2
  | ContentBlock3
  | ToolCall
  | ToolCallUpdate
  | Plan;
export type ContentBlock1 = {
  sessionUpdate: "userMessage";
} & (
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource
);
export type ContentBlock2 = {
  sessionUpdate: "agentMessageChunk";
} & (
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource
);
export type ContentBlock3 = {
  sessionUpdate: "agentThoughtChunk";
} & (
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource
);
export type PermissionOptionKind =
  | "allowOnce"
  | "allowAlways"
  | "rejectOnce"
  | "rejectAlways";
export type ToolCallContent = ContentBlock4 | Diff;
export type ContentBlock4 =
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource;
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
 * Text provided to or from an LLM.
 */
export interface TextContent {
  type: "text";
}
/**
 * An image provided to or from an LLM.
 */
export interface ImageContent {
  type: "image";
}
/**
 * Audio provided to or from an LLM.
 */
export interface AudioContent {
  type: "audio";
}
/**
 * A resource that the server is capable of reading, included in a prompt or tool call result.
 *
 * Note: resource links returned by tools are not guaranteed to appear in the results of `resources/list` requests.
 */
export interface ResourceLink {
  type: "resource_link";
}
/**
 * The contents of a resource, embedded into a prompt or tool call result.
 *
 * It is up to the client how best to render embedded resources for the benefit
 * of the LLM and/or the user.
 */
export interface EmbeddedResource {
  type: "resource";
}
export interface ToolCall {
  sessionUpdate: "toolCall";
}
export interface ToolCallUpdate {
  sessionUpdate: "toolCallUpdate";
}
export interface Plan {
  sessionUpdate: "plan";
}
export interface RequestPermissionArguments {
  options: PermissionOption[];
  sessionId: string;
  toolCall: ToolCall1;
}
export interface PermissionOption {
  kind: PermissionOptionKind;
  label: string;
  optionId: string;
}
export interface ToolCall1 {
  content?: ToolCallContent[];
  kind: ToolKind;
  label: string;
  locations?: ToolCallLocation[];
  rawInput?: unknown;
  status: ToolCallStatus;
  toolCallId: string;
}
export interface Diff {
  diff: Diff1;
}
export interface Diff1 {
  newText: string;
  oldText: string | null;
  path: string;
}
export interface ToolCallLocation {
  line?: number | null;
  path: string;
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
