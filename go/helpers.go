package acp

// TextBlock constructs a text content block.
func TextBlock(text string) ContentBlock {
	return ContentBlock{Text: &ContentBlockText{
		Text: text,
		Type: "text",
	}}
}

// ImageBlock constructs an inline image content block with base64-encoded data.
func ImageBlock(data string, mimeType string) ContentBlock {
	return ContentBlock{Image: &ContentBlockImage{
		Data:     data,
		MimeType: mimeType,
		Type:     "image",
	}}
}

// AudioBlock constructs an inline audio content block with base64-encoded data.
func AudioBlock(data string, mimeType string) ContentBlock {
	return ContentBlock{Audio: &ContentBlockAudio{
		Data:     data,
		MimeType: mimeType,
		Type:     "audio",
	}}
}

// ResourceLinkBlock constructs a resource_link content block with a name and URI.
func ResourceLinkBlock(name string, uri string) ContentBlock {
	return ContentBlock{ResourceLink: &ContentBlockResourceLink{
		Name: name,
		Type: "resource_link",
		Uri:  uri,
	}}
}

// ResourceBlock wraps an embedded resource as a content block.
func ResourceBlock(res EmbeddedResource) ContentBlock {
	return ContentBlock{Resource: &ContentBlockResource{
		Resource: res.Resource,
		Type:     "resource",
	}}
}

// ToolContent wraps a content block as tool-call content.
func ToolContent(block ContentBlock) ToolCallContent {
	return ToolCallContent{Content: &ToolCallContentContent{
		Content: block,
		Type:    "content",
	}}
}

// ToolDiffContent constructs a diff tool-call content. If oldText is omitted, the field is left empty.
func ToolDiffContent(path string, newText string, oldText ...string) ToolCallContent {
	var o *string
	if len(oldText) > 0 {
		o = &oldText[0]
	}
	return ToolCallContent{Diff: &ToolCallContentDiff{
		NewText: newText,
		OldText: o,
		Path:    path,
		Type:    "diff",
	}}
}

// ToolTerminalRef constructs a terminal reference tool-call content.
func ToolTerminalRef(terminalID string) ToolCallContent {
	return ToolCallContent{Terminal: &ToolCallContentTerminal{
		TerminalId: terminalID,
		Type:       "terminal",
	}}
}

// Ptr returns a pointer to v.
func Ptr[T any](v T) *T {
	return &v
}

// UpdateUserMessage constructs a user_message_chunk update with the given content.
func UpdateUserMessage(content ContentBlock) SessionUpdate {
	return SessionUpdate{UserMessageChunk: &SessionUpdateUserMessageChunk{Content: content}}
}

// UpdateUserMessageText constructs a user_message_chunk update from text.
func UpdateUserMessageText(text string) SessionUpdate {
	return UpdateUserMessage(TextBlock(text))
}

// UpdateAgentMessage constructs an agent_message_chunk update with the given content.
func UpdateAgentMessage(content ContentBlock) SessionUpdate {
	return SessionUpdate{AgentMessageChunk: &SessionUpdateAgentMessageChunk{Content: content}}
}

// UpdateAgentMessageText constructs an agent_message_chunk update from text.
func UpdateAgentMessageText(text string) SessionUpdate {
	return UpdateAgentMessage(TextBlock(text))
}

// UpdateAgentThought constructs an agent_thought_chunk update with the given content.
func UpdateAgentThought(content ContentBlock) SessionUpdate {
	return SessionUpdate{AgentThoughtChunk: &SessionUpdateAgentThoughtChunk{Content: content}}
}

// UpdateAgentThoughtText constructs an agent_thought_chunk update from text.
func UpdateAgentThoughtText(text string) SessionUpdate {
	return UpdateAgentThought(TextBlock(text))
}

// UpdatePlan constructs a plan update with the provided entries.
func UpdatePlan(entries ...PlanEntry) SessionUpdate {
	return SessionUpdate{Plan: &SessionUpdatePlan{Entries: entries}}
}

type ToolCallStartOpt func(tc *SessionUpdateToolCall)

// StartToolCall constructs a tool_call update with required fields and applies optional modifiers.
func StartToolCall(id ToolCallId, title string, opts ...ToolCallStartOpt) SessionUpdate {
	tc := SessionUpdateToolCall{
		Title:      title,
		ToolCallId: id,
	}
	for _, opt := range opts {
		opt(&tc)
	}
	return SessionUpdate{ToolCall: &tc}
}

// WithStartKind sets the kind for a tool_call start update.
func WithStartKind(k ToolKind) ToolCallStartOpt {
	return func(tc *SessionUpdateToolCall) {
		tc.Kind = k
	}
}

// WithStartStatus sets the status for a tool_call start update.
func WithStartStatus(s ToolCallStatus) ToolCallStartOpt {
	return func(tc *SessionUpdateToolCall) {
		tc.Status = s
	}
}

// WithStartContent sets the initial content for a tool_call start update.
func WithStartContent(c []ToolCallContent) ToolCallStartOpt {
	return func(tc *SessionUpdateToolCall) {
		tc.Content = c
	}
}

// WithStartLocations sets file locations and, if a single path is provided and rawInput is empty, mirrors it as rawInput.path.
func WithStartLocations(l []ToolCallLocation) ToolCallStartOpt {
	return func(tc *SessionUpdateToolCall) {
		tc.Locations = l
		if len(l) == 1 && l[0].Path != "" {
			if tc.RawInput == nil {
				tc.RawInput = map[string]any{"path": l[0].Path}
			} else {
				m, ok := tc.RawInput.(map[string]any)
				if ok {
					if _, exists := m["path"]; !exists {
						m["path"] = l[0].Path
					}
				}
			}
		}
	}
}

// WithStartRawInput sets rawInput for a tool_call start update.
func WithStartRawInput(v any) ToolCallStartOpt {
	return func(tc *SessionUpdateToolCall) {
		tc.RawInput = v
	}
}

// WithStartRawOutput sets rawOutput for a tool_call start update.
func WithStartRawOutput(v any) ToolCallStartOpt {
	return func(tc *SessionUpdateToolCall) {
		tc.RawOutput = v
	}
}

type ToolCallUpdateOpt func(tu *SessionUpdateToolCallUpdate)

// UpdateToolCall constructs a tool_call_update with the given ID and applies optional modifiers.
func UpdateToolCall(id ToolCallId, opts ...ToolCallUpdateOpt) SessionUpdate {
	tu := SessionUpdateToolCallUpdate{ToolCallId: id}
	for _, opt := range opts {
		opt(&tu)
	}
	return SessionUpdate{ToolCallUpdate: &tu}
}

// WithUpdateTitle sets the title for a tool_call_update.
func WithUpdateTitle(t string) ToolCallUpdateOpt {
	return func(tu *SessionUpdateToolCallUpdate) {
		tu.Title = Ptr(t)
	}
}

// WithUpdateKind sets the kind for a tool_call_update.
func WithUpdateKind(k ToolKind) ToolCallUpdateOpt {
	return func(tu *SessionUpdateToolCallUpdate) {
		tu.Kind = Ptr(k)
	}
}

// WithUpdateStatus sets the status for a tool_call_update.
func WithUpdateStatus(s ToolCallStatus) ToolCallUpdateOpt {
	return func(tu *SessionUpdateToolCallUpdate) {
		tu.Status = Ptr(s)
	}
}

// WithUpdateContent replaces the content collection for a tool_call_update.
func WithUpdateContent(c []ToolCallContent) ToolCallUpdateOpt {
	return func(tu *SessionUpdateToolCallUpdate) {
		tu.Content = c
	}
}

// WithUpdateLocations replaces the locations collection for a tool_call_update.
func WithUpdateLocations(l []ToolCallLocation) ToolCallUpdateOpt {
	return func(tu *SessionUpdateToolCallUpdate) {
		tu.Locations = l
	}
}

// WithUpdateRawInput sets rawInput for a tool_call_update.
func WithUpdateRawInput(v any) ToolCallUpdateOpt {
	return func(tu *SessionUpdateToolCallUpdate) {
		tu.RawInput = v
	}
}

// WithUpdateRawOutput sets rawOutput for a tool_call_update.
func WithUpdateRawOutput(v any) ToolCallUpdateOpt {
	return func(tu *SessionUpdateToolCallUpdate) {
		tu.RawOutput = v
	}
}

// StartReadToolCall constructs a 'tool_call' update for reading a file: kind=read, status=pending, locations=[{path}], rawInput={path}.
func StartReadToolCall(id ToolCallId, title string, path string, opts ...ToolCallStartOpt) SessionUpdate {
	base := []ToolCallStartOpt{WithStartKind(ToolKindRead), WithStartStatus(ToolCallStatusPending), WithStartLocations([]ToolCallLocation{{Path: path}}), WithStartRawInput(map[string]any{"path": path})}
	args := append(base, opts...)
	return StartToolCall(id, title, args...)
}

// StartEditToolCall constructs a 'tool_call' update for editing content: kind=edit, status=pending, locations=[{path}], rawInput={path, content}.
func StartEditToolCall(id ToolCallId, title string, path string, content any, opts ...ToolCallStartOpt) SessionUpdate {
	base := []ToolCallStartOpt{WithStartKind(ToolKindEdit), WithStartStatus(ToolCallStatusPending), WithStartLocations([]ToolCallLocation{{Path: path}}), WithStartRawInput(map[string]any{
		"content": content,
		"path":    path,
	})}
	args := append(base, opts...)
	return StartToolCall(id, title, args...)
}
