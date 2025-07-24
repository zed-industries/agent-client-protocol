export const NEW_SESSION_TOOL_NAME = "acp__new_session";
export const LOAD_SESSION_TOOL_NAME = "acp__load_session";
export const PROMPT_TOOL_NAME = "acp__prompt";

export type AgentClientProtocol =
  | [unknown, unknown]
  | LoadSessionToolArguments
  | PromptToolArguments
  | SessionUpdate
  | WriteTextFileToolArguments
  | ReadTextFileArguments;
export type ContentBlock =
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource;
export type SessionUpdate =
  | {
      session_update: "started";
    }
  | ContentBlock1
  | ContentBlock2
  | ContentBlock3
  | ToolCall
  | ToolCallUpdate
  | Plan;
export type ContentBlock1 = {
  session_update: "userMessage";
} & (
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource
);
export type ContentBlock2 = {
  session_update: "agentMessageChunk";
} & (
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource
);
export type ContentBlock3 = {
  session_update: "agentThoughtChunk";
} & (
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource
);

export interface LoadSessionToolArguments {
  clientTools: ClientTools;
  cwd: string;
  mcpServers: {
    [k: string]: McpServerConfig;
  };
  sessionId: string;
}
export interface ClientTools {
  confirmPermission: McpToolId | null;
  readTextFile: McpToolId | null;
  writeTextFile: McpToolId | null;
}
export interface McpToolId {
  mcpServer: string;
  toolName: string;
}
export interface McpServerConfig {
  args: string[];
  command: string;
  /**
   * If provided, only the specified tools are enabled
   */
  enabledTools: string[] | null;
  env?: {
    [k: string]: string;
  } | null;
}
export interface PromptToolArguments {
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
  session_update: "toolCall";
}
export interface ToolCallUpdate {
  session_update: "toolCallUpdate";
}
export interface Plan {
  session_update: "plan";
}
export interface WriteTextFileToolArguments {
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
