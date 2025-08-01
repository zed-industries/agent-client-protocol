export const AGENT_METHODS = {
  authenticate: "authenticate",
  session_new: "session/new",
  session_load: "session/load",
  session_prompt: "session/prompt",
  session_update: "session/update",
};

export const CLIENT_METHODS = {
  session_request_permission: "session/request_permission",
  session_cancelled: "session/cancelled",
  fs_write_text_file: "fs/write_text_file",
  fs_read_text_file: "fs/read_text_file",
};

import { z } from "zod";

export type WriteTextFileRequest = z.infer<typeof writeTextFileRequestSchema>;

export type ReadTextFileRequest = z.infer<typeof readTextFileRequestSchema>;

export type PermissionOptionKind = z.infer<typeof permissionOptionKindSchema>;

export type Role = z.infer<typeof roleSchema>;

export type TextResourceContents = z.infer<typeof textResourceContentsSchema>;

export type BlobResourceContents = z.infer<typeof blobResourceContentsSchema>;

export type ToolKind = z.infer<typeof toolKindSchema>;

export type ToolCallStatus = z.infer<typeof toolCallStatusSchema>;

export type WriteTextFileResponse = z.infer<typeof writeTextFileResponseSchema>;

export type ReadTextFileResponse = z.infer<typeof readTextFileResponseSchema>;

export type RequestPermissionOutcome = z.infer<
  typeof requestPermissionOutcomeSchema
>;

export type CancelledNotification = z.infer<typeof cancelledNotificationSchema>;

export type AuthenticateRequest = z.infer<typeof authenticateRequestSchema>;

export type AuthenticateResponse = z.infer<typeof authenticateResponseSchema>;

export type PromptResponse = z.infer<typeof promptResponseSchema>;

export type ToolCall1 = z.infer<typeof toolCall1Schema>;

export type ToolCallUpdate = z.infer<typeof toolCallUpdateSchema>;

export type Plan = z.infer<typeof planSchema>;

export type PermissionOption = z.infer<typeof permissionOptionSchema>;

export type ToolCallLocation = z.infer<typeof toolCallLocationSchema>;

export type Annotations = z.infer<typeof annotationsSchema>;

export type RequestPermissionResponse = z.infer<
  typeof requestPermissionResponseSchema
>;

export type EnvVariable = z.infer<typeof envVariableSchema>;

export type McpServer = z.infer<typeof mcpServerSchema>;

export type AuthMethod = z.infer<typeof authMethodSchema>;

export type LoadSessionResponse = z.infer<typeof loadSessionResponseSchema>;

export type ClientResponse = z.infer<typeof clientResponseSchema>;

export type ClientNotification = z.infer<typeof clientNotificationSchema>;

export type EmbeddedResourceResource = z.infer<
  typeof embeddedResourceResourceSchema
>;

export type NewSessionRequest = z.infer<typeof newSessionRequestSchema>;

export type LoadSessionRequest = z.infer<typeof loadSessionRequestSchema>;

export type NewSessionResponse = z.infer<typeof newSessionResponseSchema>;

export type ContentBlock = z.infer<typeof contentBlockSchema>;

export type ToolCallContent = z.infer<typeof toolCallContentSchema>;

export type PromptRequest = z.infer<typeof promptRequestSchema>;

export type AgentRequest = z.infer<typeof agentRequestSchema>;

export type AgentResponse = z.infer<typeof agentResponseSchema>;

export type SessionNotification = z.infer<typeof sessionNotificationSchema>;

export type ToolCall = z.infer<typeof toolCallSchema>;

export type AgentNotification = z.infer<typeof agentNotificationSchema>;

export type RequestPermissionRequest = z.infer<
  typeof requestPermissionRequestSchema
>;

export type ClientRequest = z.infer<typeof clientRequestSchema>;

export type AgentClientProtocol = z.infer<typeof agentClientProtocolSchema>;

export const writeTextFileRequestSchema = z.object({
  content: z.string(),
  path: z.string(),
  sessionId: z.string(),
});

export const readTextFileRequestSchema = z.object({
  limit: z.number().optional().nullable(),
  line: z.number().optional().nullable(),
  path: z.string(),
  sessionId: z.string(),
});

export const permissionOptionKindSchema = z.union([
  z.literal("allowOnce"),
  z.literal("allowAlways"),
  z.literal("rejectOnce"),
  z.literal("rejectAlways"),
]);

export const roleSchema = z.union([z.literal("assistant"), z.literal("user")]);

export const textResourceContentsSchema = z.object({
  mimeType: z.string().optional().nullable(),
  text: z.string(),
  uri: z.string(),
});

export const blobResourceContentsSchema = z.object({
  blob: z.string(),
  mimeType: z.string().optional().nullable(),
  uri: z.string(),
});

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

export const toolCallStatusSchema = z.union([
  z.literal("pending"),
  z.literal("inProgress"),
  z.literal("completed"),
  z.literal("failed"),
]);

export const writeTextFileResponseSchema = z.null();

export const readTextFileResponseSchema = z.object({
  content: z.string(),
});

export const requestPermissionOutcomeSchema = z.union([
  z.object({
    outcome: z.literal("cancelled"),
  }),
  z.object({
    optionId: z.string(),
    outcome: z.literal("selected"),
  }),
]);

export const cancelledNotificationSchema = z.object({
  sessionId: z.string(),
});

export const authenticateRequestSchema = z.object({
  methodId: z.string(),
});

export const authenticateResponseSchema = z.null();

export const promptResponseSchema = z.null();

export const toolCall1Schema = z.object({
  sessionUpdate: z.literal("toolCall"),
});

export const toolCallUpdateSchema = z.object({
  sessionUpdate: z.literal("toolCallUpdate"),
});

export const planSchema = z.object({
  sessionUpdate: z.literal("plan"),
});

export const permissionOptionSchema = z.object({
  kind: permissionOptionKindSchema,
  label: z.string(),
  optionId: z.string(),
});

export const toolCallLocationSchema = z.object({
  line: z.number().optional().nullable(),
  path: z.string(),
});

export const annotationsSchema = z.object({
  audience: z.array(roleSchema).optional().nullable(),
  lastModified: z.string().optional().nullable(),
  priority: z.number().optional().nullable(),
});

export const requestPermissionResponseSchema = z.object({
  outcome: requestPermissionOutcomeSchema,
});

export const envVariableSchema = z.object({
  name: z.string(),
  value: z.string(),
});

export const mcpServerSchema = z.object({
  args: z.array(z.string()),
  command: z.string(),
  env: z.array(envVariableSchema),
  name: z.string(),
});

export const authMethodSchema = z.object({
  description: z.string().nullable(),
  id: z.string(),
  label: z.string(),
});

export const loadSessionResponseSchema = z.object({
  authMethods: z.array(authMethodSchema),
  authRequired: z.boolean(),
});

export const clientResponseSchema = z.union([
  writeTextFileResponseSchema,
  readTextFileResponseSchema,
  requestPermissionResponseSchema,
]);

export const clientNotificationSchema = cancelledNotificationSchema;

export const embeddedResourceResourceSchema = z.union([
  textResourceContentsSchema,
  blobResourceContentsSchema,
]);

export const newSessionRequestSchema = z.object({
  cwd: z.string(),
  mcpServers: z.array(mcpServerSchema),
});

export const loadSessionRequestSchema = z.object({
  cwd: z.string(),
  mcpServers: z.array(mcpServerSchema),
  sessionId: z.string(),
});

export const newSessionResponseSchema = z.object({
  authMethods: z.array(authMethodSchema),
  sessionId: z.string().nullable(),
});

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

export const toolCallContentSchema = z.union([
  z.object({
    content: contentBlockSchema,
    type: z.literal("content"),
  }),
  z.object({
    newText: z.string(),
    oldText: z.string().nullable(),
    path: z.string(),
    type: z.literal("diff"),
  }),
]);

export const promptRequestSchema = z.object({
  prompt: z.array(contentBlockSchema),
  sessionId: z.string(),
});

export const agentRequestSchema = z.union([
  authenticateRequestSchema,
  newSessionRequestSchema,
  loadSessionRequestSchema,
  promptRequestSchema,
]);

export const agentResponseSchema = z.union([
  authenticateResponseSchema,
  newSessionResponseSchema,
  loadSessionResponseSchema,
  promptResponseSchema,
]);

export const sessionNotificationSchema = z.union([
  z.object({
    content: contentBlockSchema,
    sessionUpdate: z.literal("userMessageChunk"),
  }),
  z.object({
    content: contentBlockSchema,
    sessionUpdate: z.literal("agentMessageChunk"),
  }),
  z.object({
    content: contentBlockSchema,
    sessionUpdate: z.literal("agentThoughtChunk"),
  }),
  toolCall1Schema,
  toolCallUpdateSchema,
  planSchema,
]);

export const toolCallSchema = z.object({
  content: z.array(toolCallContentSchema).optional(),
  kind: toolKindSchema,
  label: z.string(),
  locations: z.array(toolCallLocationSchema).optional(),
  rawInput: z.unknown().optional(),
  status: toolCallStatusSchema,
  toolCallId: z.string(),
});

export const agentNotificationSchema = sessionNotificationSchema;

export const requestPermissionRequestSchema = z.object({
  options: z.array(permissionOptionSchema),
  sessionId: z.string(),
  toolCall: toolCallSchema,
});

export const clientRequestSchema = z.union([
  writeTextFileRequestSchema,
  readTextFileRequestSchema,
  requestPermissionRequestSchema,
]);

export const agentClientProtocolSchema = z.union([
  clientRequestSchema,
  clientResponseSchema,
  clientNotificationSchema,
  agentRequestSchema,
  agentResponseSchema,
  agentNotificationSchema,
]);
