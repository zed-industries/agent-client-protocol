package acp

import (
	"context"
	"os"
)

// agentExample mirrors the go/example/agent flow in a compact form.
// It streams a short message, demonstrates a tool call + permission,
// then ends the turn.
type agentExample struct{ conn *AgentSideConnection }

func (a *agentExample) SetAgentConnection(c *AgentSideConnection) { a.conn = c }

func (agentExample) Authenticate(ctx context.Context, _ AuthenticateRequest) error { return nil }
func (agentExample) Initialize(ctx context.Context, _ InitializeRequest) (InitializeResponse, error) {
	return InitializeResponse{
		ProtocolVersion:   ProtocolVersionNumber,
		AgentCapabilities: AgentCapabilities{LoadSession: false},
	}, nil
}
func (agentExample) Cancel(ctx context.Context, _ CancelNotification) error { return nil }
func (agentExample) NewSession(ctx context.Context, _ NewSessionRequest) (NewSessionResponse, error) {
	return NewSessionResponse{SessionId: SessionId("sess_demo")}, nil
}

func (a *agentExample) Prompt(ctx context.Context, p PromptRequest) (PromptResponse, error) {
	// Stream an initial agent message.
	_ = a.conn.SessionUpdate(ctx, SessionNotification{
		SessionId: p.SessionId,
		Update: SessionUpdate{
			AgentMessageChunk: &SessionUpdateAgentMessageChunk{
				Content: TextBlock("I'll help you with that."),
			},
		},
	})

	// Announce a tool call.
	_ = a.conn.SessionUpdate(ctx, SessionNotification{
		SessionId: p.SessionId,
		Update: SessionUpdate{ToolCall: &SessionUpdateToolCall{
			ToolCallId: ToolCallId("call_1"),
			Title:      "Modifying configuration",
			Kind:       ToolKindEdit,
			Status:     ToolCallStatusPending,
			Locations:  []ToolCallLocation{{Path: "/project/config.json"}},
			RawInput:   map[string]any{"path": "/project/config.json"},
		}},
	})

	// Ask the client for permission to proceed with the change.
	resp, _ := a.conn.RequestPermission(ctx, RequestPermissionRequest{
		SessionId: p.SessionId,
		ToolCall: ToolCallUpdate{
			ToolCallId: ToolCallId("call_1"),
			Title:      Ptr("Modifying configuration"),
			Kind:       Ptr(ToolKindEdit),
			Status:     Ptr(ToolCallStatusPending),
			Locations:  []ToolCallLocation{{Path: "/project/config.json"}},
			RawInput:   map[string]any{"path": "/project/config.json"},
		},
		Options: []PermissionOption{
			{Kind: PermissionOptionKindAllowOnce, Name: "Allow", OptionId: PermissionOptionId("allow")},
			{Kind: PermissionOptionKindRejectOnce, Name: "Reject", OptionId: PermissionOptionId("reject")},
		},
	})

	if resp.Outcome.Selected != nil && string(resp.Outcome.Selected.OptionId) == "allow" {
		// Mark tool call completed and stream a final message.
		_ = a.conn.SessionUpdate(ctx, SessionNotification{
			SessionId: p.SessionId,
			Update: SessionUpdate{ToolCallUpdate: &SessionUpdateToolCallUpdate{
				ToolCallId: ToolCallId("call_1"),
				Status:     ToolCallStatusCompleted,
				RawOutput:  map[string]any{"success": true},
			}},
		})
		_ = a.conn.SessionUpdate(ctx, SessionNotification{
			SessionId: p.SessionId,
			Update:    SessionUpdate{AgentMessageChunk: &SessionUpdateAgentMessageChunk{Content: TextBlock("Done.")}},
		})
	}

	return PromptResponse{StopReason: StopReasonEndTurn}, nil
}

// Example_agent wires the Agent to stdio so an external client
// can connect via this process' stdin/stdout.
func Example_agent() {
	ag := &agentExample{}
	asc := NewAgentSideConnection(ag, os.Stdout, os.Stdin)
	ag.SetAgentConnection(asc)
	// In a real program, block until the peer disconnects:
	// <-asc.Done()
}
