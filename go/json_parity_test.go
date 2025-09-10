package acp

import (
	"encoding/json"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

// normalize unmarshals both sides to generic values and compare structurally.
func equalJSON(a, b []byte) (bool, string, string) {
	var va any
	var vb any
	if err := json.Unmarshal(a, &va); err != nil {
		return false, string(a), string(b)
	}
	if err := json.Unmarshal(b, &vb); err != nil {
		return false, string(a), string(b)
	}
	return reflect.DeepEqual(va, vb), string(a), string(b)
}

func mustReadGolden(t *testing.T, name string) []byte {
	t.Helper()
	p := filepath.Join("testdata", "json_golden", name)
	b, err := os.ReadFile(p)
	if err != nil {
		t.Fatalf("read golden %s: %v", p, err)
	}
	return b
}

// Generic golden runner for a specific type T. Accepts one or more builders and
// returns a subtest function that asserts they all serialize to the same golden
// file derived from the subtest name.
func runGolden[T any](builds ...func() T) func(t *testing.T) {
	return func(t *testing.T) {
		t.Helper()
		t.Parallel()
		// Use the current subtest name; expect pattern like "<Group>/<case_name>".
		name := t.Name()
		base := name
		if i := strings.LastIndex(base, "/"); i >= 0 {
			base = base[i+1:]
		}
		want := mustReadGolden(t, base+".json")
		// Forward serialization for each builder matches the same golden JSON.
		for _, build := range builds {
			got, err := json.Marshal(build())
			if err != nil {
				t.Fatalf("marshal %s: %v", base, err)
			}
			if ok, ga, gw := equalJSON(got, want); !ok {
				t.Fatalf("%s marshal mismatch\n got: %s\nwant: %s", base, ga, gw)
			}
		}
		// Unmarshal golden into type, then marshal again and compare (one round-trip check).
		var v T
		if err := json.Unmarshal(want, &v); err != nil {
			t.Fatalf("unmarshal %s: %v", base, err)
		}
		round, err := json.Marshal(v)
		if err != nil {
			t.Fatalf("re-marshal %s: %v", base, err)
		}
		if ok, ga, gw := equalJSON(round, want); !ok {
			t.Fatalf("%s round-trip mismatch\n got: %s\nwant: %s", base, ga, gw)
		}
	}
}

func TestJSONGolden_ContentBlocks(t *testing.T) {
	t.Parallel()
	t.Run("content_text", runGolden(
		func() ContentBlock { return TextBlock("What's the weather like today?") },
	))
	t.Run("content_image", runGolden(
		func() ContentBlock { return ImageBlock("iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB...", "image/png") },
	))
	t.Run("content_audio", runGolden(
		func() ContentBlock { return AudioBlock("UklGRiQAAABXQVZFZm10IBAAAAABAAEAQB8AAAB...", "audio/wav") },
	))
	t.Run("content_resource_text", runGolden(
		func() ContentBlock {
			res := EmbeddedResourceResource{TextResourceContents: &TextResourceContents{Uri: "file:///home/user/script.py", MimeType: Ptr("text/x-python"), Text: "def hello():\n    print('Hello, world!')"}}
			return ResourceBlock(EmbeddedResource{Resource: res})
		},
	))
	t.Run("content_resource_blob", runGolden(
		func() ContentBlock {
			res := EmbeddedResourceResource{BlobResourceContents: &BlobResourceContents{Uri: "file:///home/user/document.pdf", MimeType: Ptr("application/pdf"), Blob: "<b64>"}}
			return ResourceBlock(EmbeddedResource{Resource: res})
		},
	))
	t.Run("content_resource_link", runGolden(
		func() ContentBlock {
			mt := "application/pdf"
			sz := 1024000
			return ContentBlock{ResourceLink: &ContentBlockResourceLink{Type: "resource_link", Uri: "file:///home/user/document.pdf", Name: "document.pdf", MimeType: &mt, Size: &sz}}
		},
		func() ContentBlock {
			cb := ResourceLinkBlock("document.pdf", "file:///home/user/document.pdf")
			mt := "application/pdf"
			sz := 1024000
			cb.ResourceLink.MimeType = &mt
			cb.ResourceLink.Size = &sz
			return cb
		},
	))
}

func TestJSONGolden_ToolCallContent(t *testing.T) {
	t.Parallel()
	t.Run("tool_content_content_text", runGolden(
		func() ToolCallContent { return ToolContent(TextBlock("Analysis complete. Found 3 issues.")) },
	))
	t.Run("tool_content_diff", runGolden(func() ToolCallContent {
		old := "{\n  \"debug\": false\n}"
		return ToolDiffContent("/home/user/project/src/config.json", "{\n  \"debug\": true\n}", old)
	}))
	t.Run("tool_content_diff_no_old", runGolden(
		func() ToolCallContent {
			return ToolDiffContent("/home/user/project/src/config.json", "{\n  \"debug\": true\n}")
		},
	))
	t.Run("tool_content_terminal", runGolden(
		func() ToolCallContent { return ToolTerminalRef("term_001") },
	))
}

func TestJSONGolden_RequestPermissionOutcome(t *testing.T) {
	t.Parallel()
	t.Run("permission_outcome_selected", runGolden(
		func() RequestPermissionOutcome {
			return RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{Outcome: "selected", OptionId: "allow-once"}}
		},
		func() RequestPermissionOutcome {
			return NewRequestPermissionOutcomeSelected("allow-once")
		},
	))
	t.Run("permission_outcome_cancelled", runGolden(
		func() RequestPermissionOutcome {
			return RequestPermissionOutcome{Cancelled: &RequestPermissionOutcomeCancelled{Outcome: "cancelled"}}
		},
		func() RequestPermissionOutcome { return NewRequestPermissionOutcomeCancelled() },
	))
}

func TestJSONGolden_SessionUpdates(t *testing.T) {
	t.Parallel()
	t.Run("session_update_user_message_chunk", runGolden(
		func() SessionUpdate {
			return SessionUpdate{UserMessageChunk: &SessionUpdateUserMessageChunk{Content: TextBlock("What's the capital of France?")}}
		},
		func() SessionUpdate { return UpdateUserMessageText("What's the capital of France?") },
	))
	t.Run("session_update_agent_message_chunk", runGolden(
		func() SessionUpdate {
			return SessionUpdate{AgentMessageChunk: &SessionUpdateAgentMessageChunk{Content: TextBlock("The capital of France is Paris.")}}
		},
		func() SessionUpdate { return UpdateAgentMessageText("The capital of France is Paris.") },
	))
	t.Run("session_update_agent_thought_chunk", runGolden(
		func() SessionUpdate {
			return SessionUpdate{AgentThoughtChunk: &SessionUpdateAgentThoughtChunk{Content: TextBlock("Thinking about best approach...")}}
		},
		func() SessionUpdate { return UpdateAgentThoughtText("Thinking about best approach...") },
	))
	t.Run("session_update_plan", runGolden(
		func() SessionUpdate {
			return SessionUpdate{Plan: &SessionUpdatePlan{Entries: []PlanEntry{{Content: "Check for syntax errors", Priority: PlanEntryPriorityHigh, Status: PlanEntryStatusPending}, {Content: "Identify potential type issues", Priority: PlanEntryPriorityMedium, Status: PlanEntryStatusPending}}}}
		},
		func() SessionUpdate {
			return UpdatePlan(
				PlanEntry{Content: "Check for syntax errors", Priority: PlanEntryPriorityHigh, Status: PlanEntryStatusPending},
				PlanEntry{Content: "Identify potential type issues", Priority: PlanEntryPriorityMedium, Status: PlanEntryStatusPending},
			)
		},
	))
	t.Run("session_update_tool_call", runGolden(
		func() SessionUpdate {
			return SessionUpdate{ToolCall: &SessionUpdateToolCall{ToolCallId: "call_001", Title: "Reading configuration file", Kind: ToolKindRead, Status: ToolCallStatusPending}}
		},
		func() SessionUpdate {
			return StartToolCall("call_001", "Reading configuration file", WithStartKind(ToolKindRead), WithStartStatus(ToolCallStatusPending))
		},
	))
	t.Run("session_update_tool_call_read", runGolden(
		func() SessionUpdate {
			return StartReadToolCall("call_001", "Reading configuration file", "/home/user/project/src/config.json")
		},
	))
	t.Run("session_update_tool_call_edit", runGolden(
		func() SessionUpdate {
			return StartEditToolCall("call_003", "Apply edit", "/home/user/project/src/config.json", "print('hello')")
		},
	))
	t.Run("session_update_tool_call_locations_rawinput", runGolden(
		func() SessionUpdate {
			return StartToolCall("call_lr", "Tracking file", WithStartLocations([]ToolCallLocation{{Path: "/home/user/project/src/config.json"}}))
		},
	))
	t.Run("session_update_tool_call_update_content", runGolden(
		func() SessionUpdate {
			return SessionUpdate{ToolCallUpdate: &SessionUpdateToolCallUpdate{ToolCallId: "call_001", Status: Ptr(ToolCallStatusInProgress), Content: []ToolCallContent{ToolContent(TextBlock("Found 3 configuration files..."))}}}
		},
		func() SessionUpdate {
			return UpdateToolCall("call_001", WithUpdateStatus(ToolCallStatusInProgress), WithUpdateContent([]ToolCallContent{ToolContent(TextBlock("Found 3 configuration files..."))}))
		},
	))
	t.Run("session_update_tool_call_update_more_fields", runGolden(
		func() SessionUpdate {
			return UpdateToolCall(
				"call_010",
				WithUpdateTitle("Processing changes"),
				WithUpdateKind(ToolKindEdit),
				WithUpdateStatus(ToolCallStatusCompleted),
				WithUpdateLocations([]ToolCallLocation{{Path: "/home/user/project/src/config.json"}}),
				WithUpdateRawInput(map[string]any{"path": "/home/user/project/src/config.json"}),
				WithUpdateRawOutput(map[string]any{"result": "ok"}),
				WithUpdateContent([]ToolCallContent{ToolContent(TextBlock("Edit completed."))}),
			)
		},
	))
}

func TestJSONGolden_MethodPayloads(t *testing.T) {
	t.Parallel()
	t.Run("initialize_request", runGolden(func() InitializeRequest {
		return InitializeRequest{ProtocolVersion: 1, ClientCapabilities: ClientCapabilities{Fs: FileSystemCapability{ReadTextFile: true, WriteTextFile: true}}}
	}))
	t.Run("initialize_response", runGolden(func() InitializeResponse {
		return InitializeResponse{ProtocolVersion: 1, AgentCapabilities: AgentCapabilities{LoadSession: true, PromptCapabilities: PromptCapabilities{Image: true, Audio: true, EmbeddedContext: true}}}
	}))
	t.Run("new_session_request", runGolden(func() NewSessionRequest {
		return NewSessionRequest{Cwd: "/home/user/project", McpServers: []McpServer{{Name: "filesystem", Command: "/path/to/mcp-server", Args: []string{"--stdio"}, Env: []EnvVariable{}}}}
	}))
	t.Run("new_session_response", runGolden(func() NewSessionResponse { return NewSessionResponse{SessionId: "sess_abc123def456"} }))
	t.Run("prompt_request", runGolden(func() PromptRequest {
		return PromptRequest{SessionId: "sess_abc123def456", Prompt: []ContentBlock{TextBlock("Can you analyze this code for potential issues?"), ResourceBlock(EmbeddedResource{Resource: EmbeddedResourceResource{TextResourceContents: &TextResourceContents{Uri: "file:///home/user/project/main.py", MimeType: Ptr("text/x-python"), Text: "def process_data(items):\n    for item in items:\n        print(item)"}}})}}
	}))
	t.Run("fs_read_text_file_request", runGolden(func() ReadTextFileRequest {
		line, limit := 10, 50
		return ReadTextFileRequest{SessionId: "sess_abc123def456", Path: "/home/user/project/src/main.py", Line: &line, Limit: &limit}
	}))
	t.Run("fs_read_text_file_response", runGolden(func() ReadTextFileResponse {
		return ReadTextFileResponse{Content: "def hello_world():\n    print('Hello, world!')\n"}
	}))
	t.Run("fs_write_text_file_request", runGolden(func() WriteTextFileRequest {
		return WriteTextFileRequest{SessionId: "sess_abc123def456", Path: "/home/user/project/config.json", Content: "{\n  \"debug\": true,\n  \"version\": \"1.0.0\"\n}"}
	}))
	t.Run("request_permission_request", runGolden(func() RequestPermissionRequest {
		return RequestPermissionRequest{SessionId: "sess_abc123def456", ToolCall: ToolCallUpdate{ToolCallId: "call_001"}, Options: []PermissionOption{{OptionId: "allow-once", Name: "Allow once", Kind: PermissionOptionKindAllowOnce}, {OptionId: "reject-once", Name: "Reject", Kind: PermissionOptionKindRejectOnce}}}
	}))
	t.Run("request_permission_response_selected", runGolden(func() RequestPermissionResponse {
		return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{Outcome: "selected", OptionId: "allow-once"}}}
	}))
	t.Run("cancel_notification", runGolden(func() CancelNotification { return CancelNotification{SessionId: "sess_abc123def456"} }))
}
