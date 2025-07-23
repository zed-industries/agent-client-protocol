export const NEW_SESSION_TOOL_NAME = "acp__new_session";
export const LOAD_SESSION_TOOL_NAME = "acp__load_session";
export const PROMPT_TOOL_NAME = "acp__prompt";

export type AgentCodingProtocol =
  | NewSessionToolArguments
  | LoadSessionToolArguments
  | PromptToolArguments
  | SessionUpdate
  | [unknown, unknown]
  | WriteTextFileToolArguments
  | ReadTextFileArguments;
export type SessionId = string;
export type ContentBlock =
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource;
/**
 * The sender or recipient of messages and data in a conversation.
 */
export type Role = "assistant" | "user";
export type EmbeddedResourceResource =
  | TextResourceContents
  | BlobResourceContents;
export type SessionUpdate =
  | {
      type: "started";
    }
  | ContentBlock1
  | ContentBlock2
  | ToolCall
  | Plan;
export type ContentBlock1 = (
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource
) & {
  type: "userMessage";
};
export type ContentBlock2 = (
  | TextContent
  | ImageContent
  | AudioContent
  | ResourceLink
  | EmbeddedResource
) & {
  type: "agentMessage";
};

export interface NewSessionToolArguments {
  clientTools: ClientTools;
  cwd: string;
  mcpServers: {
    [k: string]: McpServerConfig;
  };
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
export interface LoadSessionToolArguments {
  clientTools: ClientTools;
  cwd: string;
  mcpServers: {
    [k: string]: McpServerConfig;
  };
  sessionId: SessionId;
}
export interface PromptToolArguments {
  prompt: ContentBlock[];
  sessionId: SessionId;
}
/**
 * Text provided to or from an LLM.
 */
export interface TextContent {
  type: string;
  annotations?: Annotations | null;
  text: string;
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
 * An image provided to or from an LLM.
 */
export interface ImageContent {
  type: string;
  annotations?: Annotations | null;
  data: string;
  mimeType: string;
}
/**
 * Audio provided to or from an LLM.
 */
export interface AudioContent {
  type: string;
  annotations?: Annotations | null;
  data: string;
  mimeType: string;
}
/**
 * A resource that the server is capable of reading, included in a prompt or tool call result.
 *
 * Note: resource links returned by tools are not guaranteed to appear in the results of `resources/list` requests.
 */
export interface ResourceLink {
  title?: string | null;
  description?: string | null;
  type: string;
  annotations?: Annotations | null;
  mimeType?: string | null;
  name: string;
  size?: number | null;
  uri: string;
}
/**
 * The contents of a resource, embedded into a prompt or tool call result.
 *
 * It is up to the client how best to render embedded resources for the benefit
 * of the LLM and/or the user.
 */
export interface EmbeddedResource {
  type: string;
  annotations?: Annotations | null;
  resource: EmbeddedResourceResource;
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
export interface ToolCall {
  type: "toolCall";
}
export interface Plan {
  type: "plan";
}
export interface WriteTextFileToolArguments {
  content: string;
  path: string;
}
export interface ReadTextFileArguments {
  limit?: number | null;
  line?: number | null;
  path: string;
}
