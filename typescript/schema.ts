export const AGENT_METHODS = {
  authenticate: "authenticate",
  initialize: "initialize",
  session_cancel: "session/cancel",
  session_load: "session/load",
  session_new: "session/new",
  session_prompt: "session/prompt",
};

export const CLIENT_METHODS = {
  fs_read_text_file: "fs/read_text_file",
  fs_write_text_file: "fs/write_text_file",
  session_request_permission: "session/request_permission",
  session_update: "session/update",
};

export const PROTOCOL_VERSION = 1;

import { z } from "zod";

export type WriteTextFileRequest = z.infer<typeof writeTextFileRequestSchema>;

export type ReadTextFileRequest = z.infer<typeof readTextFileRequestSchema>;

export type Role = z.infer<typeof roleSchema>;

export type TextResourceContents = z.infer<typeof textResourceContentsSchema>;

export type BlobResourceContents = z.infer<typeof blobResourceContentsSchema>;

export type ToolKind = z.infer<typeof toolKindSchema>;

export type ToolCallStatus = z.infer<typeof toolCallStatusSchema>;

export type WriteTextFileResponse = z.infer<typeof writeTextFileResponseSchema>;

export type ReadTextFileResponse = z.infer<typeof readTextFileResponseSchema>;

export type RequestPermissionResponse = z.infer<
  typeof requestPermissionResponseSchema
>;

export type CancelNotification = z.infer<typeof cancelNotificationSchema>;

export type AuthenticateRequest = z.infer<typeof authenticateRequestSchema>;

export type Annotations = z.infer<typeof annotationsSchema>;

export type EmbeddedResourceResource = z.infer<
  typeof embeddedResourceResourceSchema
>;

export type AuthenticateResponse = z.infer<typeof authenticateResponseSchema>;

export type NewSessionResponse = z.infer<typeof newSessionResponseSchema>;

export type LoadSessionResponse = z.infer<typeof loadSessionResponseSchema>;

export type PromptResponse = z.infer<typeof promptResponseSchema>;

export type PermissionOption = z.infer<typeof permissionOptionSchema>;

export type ToolCallContent = z.infer<typeof toolCallContentSchema>;

export type ToolCallLocation = z.infer<typeof toolCallLocationSchema>;

export type FileSystemCapability = z.infer<typeof fileSystemCapabilitySchema>;

export type EnvVariable = z.infer<typeof envVariableSchema>;

export type McpServer = z.infer<typeof mcpServerSchema>;

export type ContentBlock = z.infer<typeof contentBlockSchema>;

export type AuthMethod = z.infer<typeof authMethodSchema>;

export type PromptCapabilities = z.infer<typeof promptCapabilitiesSchema>;

export type PlanEntry = z.infer<typeof planEntrySchema>;

export type ClientResponse = z.infer<typeof clientResponseSchema>;

export type ClientNotification = z.infer<typeof clientNotificationSchema>;

export type NewSessionRequest = z.infer<typeof newSessionRequestSchema>;

export type LoadSessionRequest = z.infer<typeof loadSessionRequestSchema>;

export type PromptRequest = z.infer<typeof promptRequestSchema>;

export type SessionNotification = z.infer<typeof sessionNotificationSchema>;

export type ToolCallUpdate = z.infer<typeof toolCallUpdateSchema>;

export type ClientCapabilities = z.infer<typeof clientCapabilitiesSchema>;

export type AgentCapabilities = z.infer<typeof agentCapabilitiesSchema>;

export type AgentNotification = z.infer<typeof agentNotificationSchema>;

export type RequestPermissionRequest = z.infer<
  typeof requestPermissionRequestSchema
>;

export type InitializeRequest = z.infer<typeof initializeRequestSchema>;

export type InitializeResponse = z.infer<typeof initializeResponseSchema>;

export type ClientRequest = z.infer<typeof clientRequestSchema>;

export type AgentRequest = z.infer<typeof agentRequestSchema>;

export type AgentResponse = z.infer<typeof agentResponseSchema>;

export type AgentClientProtocol = z.infer<typeof agentClientProtocolSchema>;

/**
 * Request to write content to a text file.
 *
 * Only available if the client supports the `fs.writeTextFile` capability.
 */
export const writeTextFileRequestSchema = z.object({
  /**
   * The text content to write to the file.
   */
  content: z.string(),
  /**
   * Absolute path to the file to write.
   */
  path: z.string(),
  /**
   * The session ID for this request.
   */
  sessionId: z.string(),
});

/**
 * Request to read content from a text file.
 *
 * Only available if the client supports the `fs.readTextFile` capability.
 */
export const readTextFileRequestSchema = z.object({
  /**
   * Optional maximum number of lines to read.
   */
  limit: z.number().optional().nullable(),
  /**
   * Optional line number to start reading from (1-based).
   */
  line: z.number().optional().nullable(),
  /**
   * Absolute path to the file to read.
   */
  path: z.string(),
  /**
   * The session ID for this request.
   */
  sessionId: z.string(),
});

/**
 * The sender or recipient of messages and data in a conversation.
 */
export const roleSchema = z.union([z.literal("assistant"), z.literal("user")]);

/**
 * Text-based resource contents.
 */
export const textResourceContentsSchema = z.object({
  mimeType: z.string().optional().nullable(),
  text: z.string(),
  uri: z.string(),
});

/**
 * Binary resource contents.
 */
export const blobResourceContentsSchema = z.object({
  blob: z.string(),
  mimeType: z.string().optional().nullable(),
  uri: z.string(),
});

/**
 * Categories of tools that can be invoked.
 *
 * Tool kinds help clients choose appropriate icons and optimize how they
 * display tool execution progress.
 *
 * See: [https://agentclientprotocol.com/protocol/tool-calls#creating](https://agentclientprotocol.com/protocol/tool-calls#creating)
 */
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

/**
 * Execution status of a tool call.
 *
 * Tool calls progress through different statuses during their lifecycle.
 *
 * See: [https://agentclientprotocol.com/protocol/tool-calls#status](https://agentclientprotocol.com/protocol/tool-calls#status)
 */
export const toolCallStatusSchema = z.union([
  z.literal("pending"),
  z.literal("in_progress"),
  z.literal("completed"),
  z.literal("failed"),
]);

export const writeTextFileResponseSchema = z.null();

/**
 * Response containing the contents of a text file.
 */
export const readTextFileResponseSchema = z.object({
  content: z.string(),
});

/**
 * Response to a permission request.
 */
export const requestPermissionResponseSchema = z.object({
  /**
   * The user's decision on the permission request.
   */
  outcome: z.union([
    z.object({
      outcome: z.literal("cancelled"),
    }),
    z.object({
      /**
       * The ID of the option the user selected.
       */
      optionId: z.string(),
      outcome: z.literal("selected"),
    }),
  ]),
});

/**
 * Notification to cancel ongoing operations for a session.
 *
 * See: [https://agentclientprotocol.com/protocol/prompt-turn#cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
 */
export const cancelNotificationSchema = z.object({
  /**
   * The ID of the session to cancel operations for.
   */
  sessionId: z.string(),
});

/**
 * Request parameters for the authenticate method.
 *
 * Specifies which authentication method to use.
 */
export const authenticateRequestSchema = z.object({
  /**
   * The ID of the authentication method to use.
   * Must be one of the methods advertised in the initialize response.
   */
  methodId: z.string(),
});

/**
 * Optional annotations for the client. The client can use annotations to inform how objects are used or displayed
 */
export const annotationsSchema = z.object({
  audience: z.array(roleSchema).optional().nullable(),
  lastModified: z.string().optional().nullable(),
  priority: z.number().optional().nullable(),
});

/**
 * Resource content that can be embedded in a message.
 */
export const embeddedResourceResourceSchema = z.union([
  textResourceContentsSchema,
  blobResourceContentsSchema,
]);

export const authenticateResponseSchema = z.null();

/**
 * Response from creating a new session.
 *
 * See: [https://agentclientprotocol.com/protocol/session-setup#creating-a-session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 */
export const newSessionResponseSchema = z.object({
  /**
   * Unique identifier for the created session.
   * Used in all subsequent requests for this conversation.
   */
  sessionId: z.string(),
});

export const loadSessionResponseSchema = z.null();

/**
 * Response from processing a user prompt.
 *
 * See: [https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion](https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion)
 */
export const promptResponseSchema = z.object({
  /**
   * Indicates why the agent stopped processing the turn.
   */
  stopReason: z.union([
    z.literal("end_turn"),
    z.literal("max_tokens"),
    z.literal("max_turn_requests"),
    z.literal("refusal"),
    z.literal("cancelled"),
  ]),
});

/**
 * An option presented to the user when requesting permission.
 */
export const permissionOptionSchema = z.object({
  /**
   * Hint about the nature of this permission option.
   */
  kind: z.union([
    z.literal("allow_once"),
    z.literal("allow_always"),
    z.literal("reject_once"),
    z.literal("reject_always"),
  ]),
  /**
   * Human-readable label to display to the user.
   */
  name: z.string(),
  /**
   * Unique identifier for this permission option.
   */
  optionId: z.string(),
});

/**
 * Content produced by a tool call.
 *
 * Tool calls can produce different types of content including
 * standard content blocks (text, images) or file diffs.
 *
 * See: [https://agentclientprotocol.com/protocol/tool-calls#content](https://agentclientprotocol.com/protocol/tool-calls#content)
 */
export const toolCallContentSchema = z.union([
  z.object({
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
     * See: [https://agentclientprotocol.com/protocol/content](https://agentclientprotocol.com/protocol/content)
     */
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
    /**
     * The new content after modification.
     */
    newText: z.string(),
    /**
     * The original content (None for new files).
     */
    oldText: z.string().optional().nullable(),
    /**
     * The file path being modified.
     */
    path: z.string(),
    type: z.literal("diff"),
  }),
]);

/**
 * A file location being accessed or modified by a tool.
 *
 * Enables clients to implement "follow-along" features that track
 * which files the agent is working with in real-time.
 *
 * See: [https://agentclientprotocol.com/protocol/tool-calls#following-the-agent](https://agentclientprotocol.com/protocol/tool-calls#following-the-agent)
 */
export const toolCallLocationSchema = z.object({
  /**
   * Optional line number within the file.
   */
  line: z.number().optional().nullable(),
  /**
   * The file path being accessed or modified.
   */
  path: z.string(),
});

/**
 * File system capabilities supported by the client.
 * Determines which file operations the agent can request.
 */
export const fileSystemCapabilitySchema = z.object({
  /**
   * Whether the Client supports `fs/read_text_file` requests.
   */
  readTextFile: z.boolean().optional(),
  /**
   * Whether the Client supports `fs/write_text_file` requests.
   */
  writeTextFile: z.boolean().optional(),
});

/**
 * An environment variable to set when launching an MCP server.
 */
export const envVariableSchema = z.object({
  /**
   * The name of the environment variable.
   */
  name: z.string(),
  /**
   * The value to set for the environment variable.
   */
  value: z.string(),
});

/**
 * Configuration for connecting to an MCP (Model Context Protocol) server.
 *
 * MCP servers provide tools and context that the agent can use when
 * processing prompts.
 *
 * See: [https://agentclientprotocol.com/protocol/session-setup#mcp-servers](https://agentclientprotocol.com/protocol/session-setup#mcp-servers)
 */
export const mcpServerSchema = z.object({
  /**
   * Command-line arguments to pass to the MCP server.
   */
  args: z.array(z.string()),
  /**
   * Path to the MCP server executable.
   */
  command: z.string(),
  /**
   * Environment variables to set when launching the MCP server.
   */
  env: z.array(envVariableSchema),
  /**
   * Human-readable name identifying this MCP server.
   */
  name: z.string(),
});

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
 * See: [https://agentclientprotocol.com/protocol/content](https://agentclientprotocol.com/protocol/content)
 */
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

/**
 * Describes an available authentication method.
 */
export const authMethodSchema = z.object({
  /**
   * Optional description providing more details about this authentication method.
   */
  description: z.string().optional().nullable(),
  /**
   * Unique identifier for this authentication method.
   */
  id: z.string(),
  /**
   * Human-readable name of the authentication method.
   */
  name: z.string(),
});

/**
 * Prompt capabilities supported by the agent.
 */
export const promptCapabilitiesSchema = z.object({
  /**
   * Agent supports [`ContentBlock::Audio`].
   */
  audio: z.boolean().optional(),
  /**
   * Agent supports embedded context in `session/prompt` requests.
   *
   * When enabled, the Client is allowed to include [`ContentBlock::Resource`]
   * in prompt requests for pieces of context that are referenced in the message.
   */
  embeddedContext: z.boolean().optional(),
  /**
   * Agent supports [`ContentBlock::Image`].
   */
  image: z.boolean().optional(),
});

/**
 * A single entry in the execution plan.
 *
 * Represents a task or goal that the assistant intends to accomplish
 * as part of fulfilling the user's request.
 * See: [https://agentclientprotocol.com/protocol/agent-plan#plan-entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
 */
export const planEntrySchema = z.object({
  /**
   * Human-readable description of what this task aims to accomplish.
   */
  content: z.string(),
  /**
   * The relative importance of this task.
   * Used to indicate which tasks are most critical to the overall goal.
   */
  priority: z.union([z.literal("high"), z.literal("medium"), z.literal("low")]),
  /**
   * Current execution status of this task.
   */
  status: z.union([
    z.literal("pending"),
    z.literal("in_progress"),
    z.literal("completed"),
  ]),
});

/**
 * All possible responses that a client can send to an agent.
 *
 * This enum is used internally for routing RPC responses. You typically won't need
 * to use this directly - the responses are handled automatically by the connection.
 *
 * These are responses to the corresponding AgentRequest variants.
 */
export const clientResponseSchema = z.union([
  writeTextFileResponseSchema,
  readTextFileResponseSchema,
  requestPermissionResponseSchema,
]);

/**
 * All possible notifications that a client can send to an agent.
 *
 * This enum is used internally for routing RPC notifications. You typically won't need
 * to use this directly - use the notification methods on the [`Agent`] trait instead.
 *
 * Notifications do not expect a response.
 */
export const clientNotificationSchema = cancelNotificationSchema;

/**
 * Request parameters for creating a new session.
 *
 * See: [https://agentclientprotocol.com/protocol/session-setup#creating-a-session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 */
export const newSessionRequestSchema = z.object({
  /**
   * The working directory for this session.
   * Must be an absolute path that serves as the context for file operations.
   */
  cwd: z.string(),
  /**
   * List of MCP (Model Context Protocol) servers the agent should connect to.
   * These provide tools and context to the language model.
   */
  mcpServers: z.array(mcpServerSchema),
});

/**
 * Request parameters for loading an existing session.
 *
 * Only available if the agent supports the `loadSession` capability.
 *
 * See: [https://agentclientprotocol.com/protocol/session-setup#loading-sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
 */
export const loadSessionRequestSchema = z.object({
  /**
   * The working directory for this session.
   */
  cwd: z.string(),
  /**
   * List of MCP servers to connect to for this session.
   */
  mcpServers: z.array(mcpServerSchema),
  /**
   * The ID of the session to load.
   */
  sessionId: z.string(),
});

/**
 * Request parameters for sending a user prompt to the agent.
 *
 * Contains the user's message and any additional context.
 *
 * See: [https://agentclientprotocol.com/protocol/prompt-turn#1-user-message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
 */
export const promptRequestSchema = z.object({
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
  prompt: z.array(contentBlockSchema),
  /**
   * The ID of the session to send this user message to
   */
  sessionId: z.string(),
});

/**
 * Notification containing a session update from the agent.
 *
 * Used to stream real-time progress and results during prompt processing.
 *
 * See: [https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
 */
export const sessionNotificationSchema = z.object({
  /**
   * The ID of the session this update pertains to.
   */
  sessionId: z.string(),
  /**
   * The actual update content.
   */
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
      /**
       * Content produced by the tool call.
       */
      content: z.array(toolCallContentSchema).optional(),
      /**
       * The category of tool being invoked.
       * Helps clients choose appropriate icons and UI treatment.
       */
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
      /**
       * File locations affected by this tool call.
       * Enables "follow-along" features in clients.
       */
      locations: z.array(toolCallLocationSchema).optional(),
      /**
       * Raw input parameters sent to the tool.
       */
      rawInput: z.record(z.unknown()).optional(),
      /**
       * Raw output returned by the tool.
       */
      rawOutput: z.record(z.unknown()).optional(),
      sessionUpdate: z.literal("tool_call"),
      /**
       * Current execution status of the tool call.
       */
      status: z
        .union([
          z.literal("pending"),
          z.literal("in_progress"),
          z.literal("completed"),
          z.literal("failed"),
        ])
        .optional(),
      /**
       * Human-readable title describing what the tool is doing.
       */
      title: z.string(),
      /**
       * Unique identifier for this tool call within the session.
       */
      toolCallId: z.string(),
    }),
    z.object({
      /**
       * Replace the content collection.
       */
      content: z.array(toolCallContentSchema).optional().nullable(),
      /**
       * Update the tool kind.
       */
      kind: toolKindSchema.optional().nullable(),
      /**
       * Replace the locations collection.
       */
      locations: z.array(toolCallLocationSchema).optional().nullable(),
      /**
       * Update the raw input.
       */
      rawInput: z.record(z.unknown()).optional(),
      /**
       * Update the raw output.
       */
      rawOutput: z.record(z.unknown()).optional(),
      sessionUpdate: z.literal("tool_call_update"),
      /**
       * Update the execution status.
       */
      status: toolCallStatusSchema.optional().nullable(),
      /**
       * Update the human-readable title.
       */
      title: z.string().optional().nullable(),
      /**
       * The ID of the tool call being updated.
       */
      toolCallId: z.string(),
    }),
    z.object({
      /**
       * The list of tasks to be accomplished.
       *
       * When updating a plan, the agent must send a complete list of all entries
       * with their current status. The client replaces the entire plan with each update.
       */
      entries: z.array(planEntrySchema),
      sessionUpdate: z.literal("plan"),
    }),
  ]),
});

/**
 * Details about the tool call requiring permission.
 */
export const toolCallUpdateSchema = z.object({
  /**
   * Replace the content collection.
   */
  content: z.array(toolCallContentSchema).optional().nullable(),
  /**
   * Update the tool kind.
   */
  kind: toolKindSchema.optional().nullable(),
  /**
   * Replace the locations collection.
   */
  locations: z.array(toolCallLocationSchema).optional().nullable(),
  /**
   * Update the raw input.
   */
  rawInput: z.record(z.unknown()).optional(),
  /**
   * Update the raw output.
   */
  rawOutput: z.record(z.unknown()).optional(),
  /**
   * Update the execution status.
   */
  status: toolCallStatusSchema.optional().nullable(),
  /**
   * Update the human-readable title.
   */
  title: z.string().optional().nullable(),
  /**
   * The ID of the tool call being updated.
   */
  toolCallId: z.string(),
});

/**
 * Capabilities supported by the client.
 */
export const clientCapabilitiesSchema = z.object({
  fs: fileSystemCapabilitySchema.optional(),
});

/**
 * Capabilities supported by the agent.
 */
export const agentCapabilitiesSchema = z.object({
  /**
   * Whether the agent supports `session/load`.
   */
  loadSession: z.boolean().optional(),
  promptCapabilities: promptCapabilitiesSchema.optional(),
});

/**
 * All possible notifications that an agent can send to a client.
 *
 * This enum is used internally for routing RPC notifications. You typically won't need
 * to use this directly - use the notification methods on the [`Client`] trait instead.
 *
 * Notifications do not expect a response.
 */
export const agentNotificationSchema = sessionNotificationSchema;

/**
 * Request for user permission to execute a tool call.
 *
 * Sent when the agent needs authorization before performing a sensitive operation.
 *
 * See: [https://agentclientprotocol.com/protocol/tool-calls#requesting-permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
 */
export const requestPermissionRequestSchema = z.object({
  /**
   * Available permission options for the user to choose from.
   */
  options: z.array(permissionOptionSchema),
  /**
   * The session ID for this request.
   */
  sessionId: z.string(),
  toolCall: toolCallUpdateSchema,
});

/**
 * Request parameters for the initialize method.
 *
 * Sent by the client to establish connection and negotiate capabilities.
 *
 * See: [https://agentclientprotocol.com/protocol/initialization](https://agentclientprotocol.com/protocol/initialization)
 */
export const initializeRequestSchema = z.object({
  clientCapabilities: clientCapabilitiesSchema.optional(),
  /**
   * The latest protocol version supported by the client.
   */
  protocolVersion: z.number(),
});

/**
 * Response from the initialize method.
 *
 * Contains the negotiated protocol version and agent capabilities.
 *
 * See: [https://agentclientprotocol.com/protocol/initialization](https://agentclientprotocol.com/protocol/initialization)
 */
export const initializeResponseSchema = z.object({
  agentCapabilities: agentCapabilitiesSchema.optional(),
  /**
   * Authentication methods supported by the agent.
   */
  authMethods: z.array(authMethodSchema).optional(),
  /**
   * The protocol version the client specified if supported by the agent,
   * or the latest protocol version supported by the agent.
   *
   * The client should disconnect, if it doesn't support this version.
   */
  protocolVersion: z.number(),
});

/**
 * All possible requests that an agent can send to a client.
 *
 * This enum is used internally for routing RPC requests. You typically won't need
 * to use this directly - instead, use the methods on the [`Client`] trait.
 *
 * This enum encompasses all method calls from agent to client.
 */
export const clientRequestSchema = z.union([
  writeTextFileRequestSchema,
  readTextFileRequestSchema,
  requestPermissionRequestSchema,
]);

/**
 * All possible requests that a client can send to an agent.
 *
 * This enum is used internally for routing RPC requests. You typically won't need
 * to use this directly - instead, use the methods on the [`Agent`] trait.
 *
 * This enum encompasses all method calls from client to agent.
 */
export const agentRequestSchema = z.union([
  initializeRequestSchema,
  authenticateRequestSchema,
  newSessionRequestSchema,
  loadSessionRequestSchema,
  promptRequestSchema,
]);

/**
 * All possible responses that an agent can send to a client.
 *
 * This enum is used internally for routing RPC responses. You typically won't need
 * to use this directly - the responses are handled automatically by the connection.
 *
 * These are responses to the corresponding ClientRequest variants.
 */
export const agentResponseSchema = z.union([
  initializeResponseSchema,
  authenticateResponseSchema,
  newSessionResponseSchema,
  loadSessionResponseSchema,
  promptResponseSchema,
]);

export const agentClientProtocolSchema = z.union([
  clientRequestSchema,
  clientResponseSchema,
  clientNotificationSchema,
  agentRequestSchema,
  agentResponseSchema,
  agentNotificationSchema,
]);
