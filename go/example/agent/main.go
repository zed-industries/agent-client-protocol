package main

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"os/exec"
	"os/signal"
	"sync"
	"time"

	acp "github.com/zed-industries/agent-client-protocol/go"
)

type agentSession struct {
	cancel context.CancelFunc
}

type exampleAgent struct {
	conn     *acp.AgentSideConnection
	sessions map[string]*agentSession
	mu       sync.Mutex
}

var (
	_ acp.Agent       = (*exampleAgent)(nil)
	_ acp.AgentLoader = (*exampleAgent)(nil)
)

func newExampleAgent() *exampleAgent {
	return &exampleAgent{sessions: make(map[string]*agentSession)}
}

// Implement acp.AgentConnAware to receive the connection after construction.
func (a *exampleAgent) SetAgentConnection(conn *acp.AgentSideConnection) { a.conn = conn }

func (a *exampleAgent) Initialize(ctx context.Context, params acp.InitializeRequest) (acp.InitializeResponse, error) {
	return acp.InitializeResponse{
		ProtocolVersion: acp.ProtocolVersionNumber,
		AgentCapabilities: acp.AgentCapabilities{
			LoadSession: false,
		},
	}, nil
}

func (a *exampleAgent) NewSession(ctx context.Context, params acp.NewSessionRequest) (acp.NewSessionResponse, error) {
	sid := randomID()
	a.mu.Lock()
	a.sessions[sid] = &agentSession{}
	a.mu.Unlock()
	return acp.NewSessionResponse{SessionId: acp.SessionId(sid)}, nil
}

func (a *exampleAgent) Authenticate(ctx context.Context, _ acp.AuthenticateRequest) error { return nil }

func (a *exampleAgent) LoadSession(ctx context.Context, _ acp.LoadSessionRequest) error { return nil }

func (a *exampleAgent) Cancel(ctx context.Context, params acp.CancelNotification) error {
	a.mu.Lock()
	s, ok := a.sessions[string(params.SessionId)]
	a.mu.Unlock()
	if ok && s != nil && s.cancel != nil {
		s.cancel()
	}
	return nil
}

func (a *exampleAgent) Prompt(_ context.Context, params acp.PromptRequest) (acp.PromptResponse, error) {
	sid := string(params.SessionId)
	a.mu.Lock()
	s, ok := a.sessions[sid]
	a.mu.Unlock()
	if !ok {
		return acp.PromptResponse{}, fmt.Errorf("session %s not found", sid)
	}

	// cancel any previous turn
	a.mu.Lock()
	if s.cancel != nil {
		prev := s.cancel
		a.mu.Unlock()
		prev()
	} else {
		a.mu.Unlock()
	}
	ctx, cancel := context.WithCancel(context.Background())
	a.mu.Lock()
	s.cancel = cancel
	a.mu.Unlock()

	// simulate a full turn with streaming updates and a permission request
	if err := a.simulateTurn(ctx, sid); err != nil {
		if ctx.Err() != nil {
			return acp.PromptResponse{StopReason: acp.StopReasonCancelled}, nil
		}
		return acp.PromptResponse{}, err
	}
	a.mu.Lock()
	s.cancel = nil
	a.mu.Unlock()
	return acp.PromptResponse{StopReason: acp.StopReasonEndTurn}, nil
}

func (a *exampleAgent) simulateTurn(ctx context.Context, sid string) error {
	// disclaimer: stream a demo notice so clients see it's the example agent
	if err := a.conn.SessionUpdate(ctx, acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update:    acp.UpdateAgentMessageText("ACP Go Example Agent — demo only (no AI model)."),
	}); err != nil {
		return err
	}
	if err := pause(ctx, 250*time.Millisecond); err != nil {
		return err
	}
	// initial message chunk
	if err := a.conn.SessionUpdate(ctx, acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update:    acp.UpdateAgentMessageText("I'll help you with that. Let me start by reading some files to understand the current situation."),
	}); err != nil {
		return err
	}
	if err := pause(ctx, time.Second); err != nil {
		return err
	}

	// tool call without permission
	if err := a.conn.SessionUpdate(ctx, acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update: acp.StartToolCall(
			acp.ToolCallId("call_1"),
			"Reading project files",
			acp.WithStartKind(acp.ToolKindRead),
			acp.WithStartStatus(acp.ToolCallStatusPending),
			acp.WithStartLocations([]acp.ToolCallLocation{{Path: "/project/README.md"}}),
			acp.WithStartRawInput(map[string]any{"path": "/project/README.md"}),
		),
	}); err != nil {
		return err
	}
	if err := pause(ctx, time.Second); err != nil {
		return err
	}

	// update tool call completed
	if err := a.conn.SessionUpdate(ctx, acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update: acp.UpdateToolCall(
			acp.ToolCallId("call_1"),
			acp.WithUpdateStatus(acp.ToolCallStatusCompleted),
			acp.WithUpdateContent([]acp.ToolCallContent{acp.ToolContent(acp.TextBlock("# My Project\n\nThis is a sample project..."))}),
			acp.WithUpdateRawOutput(map[string]any{"content": "# My Project\n\nThis is a sample project..."}),
		),
	}); err != nil {
		return err
	}
	if err := pause(ctx, time.Second); err != nil {
		return err
	}

	// more text
	if err := a.conn.SessionUpdate(ctx, acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update:    acp.UpdateAgentMessageText(" Now I understand the project structure. I need to make some changes to improve it."),
	}); err != nil {
		return err
	}
	if err := pause(ctx, time.Second); err != nil {
		return err
	}

	// tool call requiring permission
	if err := a.conn.SessionUpdate(ctx, acp.SessionNotification{
		SessionId: acp.SessionId(sid),
		Update: acp.StartToolCall(
			acp.ToolCallId("call_2"),
			"Modifying critical configuration file",
			acp.WithStartKind(acp.ToolKindEdit),
			acp.WithStartStatus(acp.ToolCallStatusPending),
			acp.WithStartLocations([]acp.ToolCallLocation{{Path: "/project/config.json"}}),
			acp.WithStartRawInput(map[string]any{"path": "/project/config.json", "content": "{\"database\": {\"host\": \"new-host\"}}"}),
		),
	}); err != nil {
		return err
	}

	// request permission for sensitive operation
	permResp, err := a.conn.RequestPermission(ctx, acp.RequestPermissionRequest{
		SessionId: acp.SessionId(sid),
		ToolCall: acp.ToolCallUpdate{
			ToolCallId: acp.ToolCallId("call_2"),
			Title:      acp.Ptr("Modifying critical configuration file"),
			Kind:       acp.Ptr(acp.ToolKindEdit),
			Status:     acp.Ptr(acp.ToolCallStatusPending),
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
		if err := a.conn.SessionUpdate(ctx, acp.SessionNotification{
			SessionId: acp.SessionId(sid),
			Update: acp.UpdateToolCall(
				acp.ToolCallId("call_2"),
				acp.WithUpdateStatus(acp.ToolCallStatusCompleted),
				acp.WithUpdateRawOutput(map[string]any{"success": true, "message": "Configuration updated"}),
				acp.WithUpdateTitle("Modifying critical configuration file"),
			),
		}); err != nil {
			return err
		}
		if err := pause(ctx, time.Second); err != nil {
			return err
		}
		if err := a.conn.SessionUpdate(ctx, acp.SessionNotification{
			SessionId: acp.SessionId(sid),
			Update:    acp.UpdateAgentMessageText(" Perfect! I've successfully updated the configuration. The changes have been applied."),
		}); err != nil {
			return err
		}
	case "reject":
		if err := pause(ctx, time.Second); err != nil {
			return err
		}
		if err := a.conn.SessionUpdate(ctx, acp.SessionNotification{
			SessionId: acp.SessionId(sid),
			Update:    acp.UpdateAgentMessageText(" I understand you prefer not to make that change. I'll skip the configuration update."),
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
	// If args provided, treat them as client program + args to spawn and connect via stdio.
	// Otherwise, default to stdio (allowing manual wiring or use by another process).
	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, os.Kill)
	defer cancel()

	var (
		out io.Writer = os.Stdout
		in  io.Reader = os.Stdin
		cmd *exec.Cmd
	)
	if len(os.Args) > 1 {
		cmd = exec.CommandContext(ctx, os.Args[1], os.Args[2:]...)
		cmd.Stderr = os.Stderr
		stdin, _ := cmd.StdinPipe()
		stdout, _ := cmd.StdoutPipe()
		if err := cmd.Start(); err != nil {
			fmt.Fprintf(os.Stderr, "failed to start client: %v\n", err)
			os.Exit(1)
		}
		out = stdin
		in = stdout
	}

	ag := newExampleAgent()
	asc := acp.NewAgentSideConnection(ag, out, in)
	ag.SetAgentConnection(asc)

	// Block until the peer disconnects.
	<-asc.Done()

	if cmd != nil && cmd.Process != nil {
		_ = cmd.Process.Kill()
	}
}
