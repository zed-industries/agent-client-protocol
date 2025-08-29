package main

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"time"

	acp "github.com/zed-industries/agent-client-protocol/go"
)

type agentSession struct {
	cancel context.CancelFunc
}

type exampleAgent struct {
	conn     *acp.AgentSideConnection
	sessions map[string]*agentSession
}

var _ acp.Agent = (*exampleAgent)(nil)

func newExampleAgent() *exampleAgent {
	return &exampleAgent{sessions: make(map[string]*agentSession)}
}

// Implement acp.AgentConnAware to receive the connection after construction.
func (a *exampleAgent) SetAgentConnection(conn *acp.AgentSideConnection) { a.conn = conn }

func (a *exampleAgent) Initialize(params acp.InitializeRequest) (acp.InitializeResponse, error) {
	return acp.InitializeResponse{
		ProtocolVersion: acp.ProtocolVersionNumber,
		AgentCapabilities: acp.AgentCapabilities{
			LoadSession: false,
		},
	}, nil
}

func (a *exampleAgent) NewSession(params acp.NewSessionRequest) (acp.NewSessionResponse, error) {
	sid := randomID()
	a.sessions[sid] = &agentSession{}
	return acp.NewSessionResponse{SessionId: acp.SessionId(sid)}, nil
}

func (a *exampleAgent) Authenticate(_ acp.AuthenticateRequest) error { return nil }

func (a *exampleAgent) LoadSession(_ acp.LoadSessionRequest) error { return nil }

func (a *exampleAgent) Cancel(params acp.CancelNotification) error {
	if s, ok := a.sessions[string(params.SessionId)]; ok {
		if s.cancel != nil {
			s.cancel()
		}
	}
	return nil
}

func (a *exampleAgent) Prompt(params acp.PromptRequest) (acp.PromptResponse, error) {
	sid := string(params.SessionId)
	s, ok := a.sessions[sid]
	if !ok {
		return acp.PromptResponse{}, fmt.Errorf("session %s not found", sid)
	}

	// cancel any previous turn
	if s.cancel != nil {
		s.cancel()
	}
	ctx, cancel := context.WithCancel(context.Background())
	s.cancel = cancel

	// simulate a full turn with streaming updates and a permission request
	if err := a.simulateTurn(ctx, sid); err != nil {
		if ctx.Err() != nil {
			return acp.PromptResponse{StopReason: acp.StopReasonCancelled}, nil
		}
		return acp.PromptResponse{}, err
	}
	s.cancel = nil
	return acp.PromptResponse{StopReason: acp.StopReasonEndTurn}, nil
}

func (a *exampleAgent) simulateTurn(ctx context.Context, sid string) error {
	// initial message chunk
	if err := a.conn.SessionUpdate(acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update: acp.SessionUpdate{
			AgentMessageChunk: &acp.SessionUpdateAgentMessageChunk{Content: acp.ContentBlock{
				Type: "text",
				Text: &acp.TextContent{Text: "I'll help you with that. Let me start by reading some files to understand the current situation."},
			}},
		},
	}); err != nil {
		return err
	}
	if err := pause(ctx, time.Second); err != nil {
		return err
	}

	// tool call without permission
	if err := a.conn.SessionUpdate(acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update: acp.SessionUpdate{ToolCall: &acp.SessionUpdateToolCall{
			ToolCallId: acp.ToolCallId("call_1"),
			Title:      "Reading project files",
			Kind:       acp.ToolKindRead,
			Status:     acp.ToolCallStatusPending,
			Locations:  []acp.ToolCallLocation{{Path: "/project/README.md"}},
			RawInput:   map[string]any{"path": "/project/README.md"},
		}},
	}); err != nil {
		return err
	}
	if err := pause(ctx, time.Second); err != nil {
		return err
	}

	// update tool call completed
	if err := a.conn.SessionUpdate(acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update: acp.SessionUpdate{ToolCallUpdate: &acp.SessionUpdateToolCallUpdate{
			ToolCallId: acp.ToolCallId("call_1"),
			Status:     "completed",
			Content: []acp.ToolCallContent{{
				Type:    "content",
				Content: &acp.ContentBlock{Type: "text", Text: &acp.TextContent{Text: "# My Project\n\nThis is a sample project..."}},
			}},
			RawOutput: map[string]any{"content": "# My Project\n\nThis is a sample project..."},
		}},
	}); err != nil {
		return err
	}
	if err := pause(ctx, time.Second); err != nil {
		return err
	}

	// more text
	if err := a.conn.SessionUpdate(acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update: acp.SessionUpdate{AgentMessageChunk: &acp.SessionUpdateAgentMessageChunk{Content: acp.ContentBlock{
			Type: "text",
			Text: &acp.TextContent{Text: " Now I understand the project structure. I need to make some changes to improve it."},
		}}},
	}); err != nil {
		return err
	}
	if err := pause(ctx, time.Second); err != nil {
		return err
	}

	// tool call requiring permission
	if err := a.conn.SessionUpdate(acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update: acp.SessionUpdate{ToolCall: &acp.SessionUpdateToolCall{
			ToolCallId: acp.ToolCallId("call_2"),
			Title:      "Modifying critical configuration file",
			Kind:       acp.ToolKindEdit,
			Status:     acp.ToolCallStatusPending,
			Locations:  []acp.ToolCallLocation{{Path: "/project/config.json"}},
			RawInput:   map[string]any{"path": "/project/config.json", "content": "{\"database\": {\"host\": \"new-host\"}}"},
		}},
	}); err != nil {
		return err
	}

	// request permission for sensitive operation
	permResp, err := a.conn.RequestPermission(acp.RequestPermissionRequest{
		SessionId: acp.SessionId(sid),
		ToolCall: acp.ToolCallUpdate{
			ToolCallId: acp.ToolCallId("call_2"),
			Title:      "Modifying critical configuration file",
			Kind:       "edit",
			Status:     "pending",
			Locations:  []acp.ToolCallLocation{{Path: "/home/user/project/config.json"}},
			RawInput:   map[string]any{"path": "/home/user/project/config.json", "content": "{\"database\": {\"host\": \"new-host\"}}"},
		},
		Options: []acp.PermissionOption{
			{Kind: acp.PermissionOptionKindAllowOnce, Name: "Allow this change", OptionId: acp.PermissionOptionId("allow")},
			{Kind: acp.PermissionOptionKindRejectOnce, Name: "Skip this change", OptionId: acp.PermissionOptionId("reject")},
		},
	})
	if err != nil {
		return err
	}

	// handle permission outcome
	if permResp.Outcome.Cancelled != nil {
		return nil
	}
	if permResp.Outcome.Selected == nil {
		return fmt.Errorf("unexpected permission outcome")
	}
	switch string(permResp.Outcome.Selected.OptionId) {
	case "allow":
		if err := a.conn.SessionUpdate(acp.SessionNotification{
			SessionId: acp.SessionId(sid),
			Update: acp.SessionUpdate{ToolCallUpdate: &acp.SessionUpdateToolCallUpdate{
				ToolCallId: acp.ToolCallId("call_2"),
				Status:     "completed",
				RawOutput:  map[string]any{"success": true, "message": "Configuration updated"},
			}},
		}); err != nil {
			return err
		}
		if err := pause(ctx, time.Second); err != nil {
			return err
		}
		if err := a.conn.SessionUpdate(acp.SessionNotification{
			SessionId: acp.SessionId(sid),
			Update: acp.SessionUpdate{AgentMessageChunk: &acp.SessionUpdateAgentMessageChunk{Content: acp.ContentBlock{
				Type: "text",
				Text: &acp.TextContent{Text: " Perfect! I've successfully updated the configuration. The changes have been applied."},
			}}},
		}); err != nil {
			return err
		}
	case "reject":
		if err := pause(ctx, time.Second); err != nil {
			return err
		}
		if err := a.conn.SessionUpdate(acp.SessionNotification{
			SessionId: acp.SessionId(sid),
			Update: acp.SessionUpdate{AgentMessageChunk: &acp.SessionUpdateAgentMessageChunk{Content: acp.ContentBlock{
				Type: "text",
				Text: &acp.TextContent{Text: " I understand you prefer not to make that change. I'll skip the configuration update."},
			}}},
		}); err != nil {
			return err
		}
	default:
		return fmt.Errorf("unexpected permission option: %s", permResp.Outcome.Selected.OptionId)
	}
	return nil
}

func randomID() string {
	var b [12]byte
	if _, err := io.ReadFull(rand.Reader, b[:]); err != nil {
		// fallback to time-based
		return fmt.Sprintf("sess_%d", time.Now().UnixNano())
	}
	return "sess_" + hex.EncodeToString(b[:])
}

func pause(ctx context.Context, d time.Duration) error {
	t := time.NewTimer(d)
	defer t.Stop()
	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-t.C:
		return nil
	}
}

func main() {
	// Wire up stdio: write to stdout, read from stdin
	ag := newExampleAgent()
	asc := acp.NewAgentSideConnection(ag, os.Stdout, os.Stdin)
	ag.SetAgentConnection(asc)

	// Block until the peer disconnects (stdin closes).
	<-asc.Done()
}
