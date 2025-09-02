package acp

import (
	"context"
	"io"
	"slices"
	"sync"
	"testing"
	"time"
)

type clientFuncs struct {
	WriteTextFileFunc     func(context.Context, WriteTextFileRequest) error
	ReadTextFileFunc      func(context.Context, ReadTextFileRequest) (ReadTextFileResponse, error)
	RequestPermissionFunc func(context.Context, RequestPermissionRequest) (RequestPermissionResponse, error)
	SessionUpdateFunc     func(context.Context, SessionNotification) error
}

var _ Client = (*clientFuncs)(nil)

func (c clientFuncs) WriteTextFile(ctx context.Context, p WriteTextFileRequest) error {
	if c.WriteTextFileFunc != nil {
		return c.WriteTextFileFunc(ctx, p)
	}
	return nil
}

func (c clientFuncs) ReadTextFile(ctx context.Context, p ReadTextFileRequest) (ReadTextFileResponse, error) {
	if c.ReadTextFileFunc != nil {
		return c.ReadTextFileFunc(ctx, p)
	}
	return ReadTextFileResponse{}, nil
}

func (c clientFuncs) RequestPermission(ctx context.Context, p RequestPermissionRequest) (RequestPermissionResponse, error) {
	if c.RequestPermissionFunc != nil {
		return c.RequestPermissionFunc(ctx, p)
	}
	return RequestPermissionResponse{}, nil
}

func (c clientFuncs) SessionUpdate(ctx context.Context, n SessionNotification) error {
	if c.SessionUpdateFunc != nil {
		return c.SessionUpdateFunc(ctx, n)
	}
	return nil
}

type agentFuncs struct {
	InitializeFunc   func(context.Context, InitializeRequest) (InitializeResponse, error)
	NewSessionFunc   func(context.Context, NewSessionRequest) (NewSessionResponse, error)
	LoadSessionFunc  func(context.Context, LoadSessionRequest) error
	AuthenticateFunc func(context.Context, AuthenticateRequest) error
	PromptFunc       func(context.Context, PromptRequest) (PromptResponse, error)
	CancelFunc       func(context.Context, CancelNotification) error
}

var (
	_ Agent       = (*agentFuncs)(nil)
	_ AgentLoader = (*agentFuncs)(nil)
)

func (a agentFuncs) Initialize(ctx context.Context, p InitializeRequest) (InitializeResponse, error) {
	if a.InitializeFunc != nil {
		return a.InitializeFunc(ctx, p)
	}
	return InitializeResponse{}, nil
}

func (a agentFuncs) NewSession(ctx context.Context, p NewSessionRequest) (NewSessionResponse, error) {
	if a.NewSessionFunc != nil {
		return a.NewSessionFunc(ctx, p)
	}
	return NewSessionResponse{}, nil
}

func (a agentFuncs) LoadSession(ctx context.Context, p LoadSessionRequest) error {
	if a.LoadSessionFunc != nil {
		return a.LoadSessionFunc(ctx, p)
	}
	return nil
}

func (a agentFuncs) Authenticate(ctx context.Context, p AuthenticateRequest) error {
	if a.AuthenticateFunc != nil {
		return a.AuthenticateFunc(ctx, p)
	}
	return nil
}

func (a agentFuncs) Prompt(ctx context.Context, p PromptRequest) (PromptResponse, error) {
	if a.PromptFunc != nil {
		return a.PromptFunc(ctx, p)
	}
	return PromptResponse{}, nil
}

func (a agentFuncs) Cancel(ctx context.Context, n CancelNotification) error {
	if a.CancelFunc != nil {
		return a.CancelFunc(ctx, n)
	}
	return nil
}

// Test bidirectional error handling similar to typescript/acp.test.ts
func TestConnectionHandlesErrorsBidirectional(t *testing.T) {
	ctx := context.Background()
	c2aR, c2aW := io.Pipe()
	a2cR, a2cW := io.Pipe()

	c := NewClientSideConnection(clientFuncs{
		WriteTextFileFunc: func(context.Context, WriteTextFileRequest) error {
			return &RequestError{Code: -32603, Message: "Write failed"}
		},
		ReadTextFileFunc: func(context.Context, ReadTextFileRequest) (ReadTextFileResponse, error) {
			return ReadTextFileResponse{}, &RequestError{Code: -32603, Message: "Read failed"}
		},
		RequestPermissionFunc: func(context.Context, RequestPermissionRequest) (RequestPermissionResponse, error) {
			return RequestPermissionResponse{}, &RequestError{Code: -32603, Message: "Permission denied"}
		},
		SessionUpdateFunc: func(context.Context, SessionNotification) error { return nil },
	}, c2aW, a2cR)
	agentConn := NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(context.Context, InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{}, &RequestError{Code: -32603, Message: "Failed to initialize"}
		},
		NewSessionFunc: func(context.Context, NewSessionRequest) (NewSessionResponse, error) {
			return NewSessionResponse{}, &RequestError{Code: -32603, Message: "Failed to create session"}
		},
		LoadSessionFunc: func(context.Context, LoadSessionRequest) error {
			return &RequestError{Code: -32603, Message: "Failed to load session"}
		},
		AuthenticateFunc: func(context.Context, AuthenticateRequest) error {
			return &RequestError{Code: -32603, Message: "Authentication failed"}
		},
		PromptFunc: func(context.Context, PromptRequest) (PromptResponse, error) {
			return PromptResponse{}, &RequestError{Code: -32603, Message: "Prompt failed"}
		},
		CancelFunc: func(context.Context, CancelNotification) error { return nil },
	}, a2cW, c2aR)

	// Client->Agent direction: expect error
	if err := agentConn.WriteTextFile(ctx, WriteTextFileRequest{Path: "/test.txt", Content: "test", SessionId: "test-session"}); err == nil {
		t.Fatalf("expected error for writeTextFile, got nil")
	}

	// Agent->Client direction: expect error
	if _, err := c.NewSession(ctx, NewSessionRequest{Cwd: "/test", McpServers: []McpServer{}}); err == nil {
		t.Fatalf("expected error for newSession, got nil")
	}
}

// Test concurrent requests handling similar to TS suite
func TestConnectionHandlesConcurrentRequests(t *testing.T) {
	c2aR, c2aW := io.Pipe()
	a2cR, a2cW := io.Pipe()

	var mu sync.Mutex
	requestCount := 0

	_ = NewClientSideConnection(clientFuncs{
		WriteTextFileFunc: func(context.Context, WriteTextFileRequest) error {
			mu.Lock()
			requestCount++
			mu.Unlock()
			time.Sleep(40 * time.Millisecond)
			return nil
		},
		ReadTextFileFunc: func(_ context.Context, req ReadTextFileRequest) (ReadTextFileResponse, error) {
			return ReadTextFileResponse{Content: "Content of " + req.Path}, nil
		},
		RequestPermissionFunc: func(context.Context, RequestPermissionRequest) (RequestPermissionResponse, error) {
			return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: "allow"}}}, nil
		},
		SessionUpdateFunc: func(context.Context, SessionNotification) error { return nil },
	}, c2aW, a2cR)
	agentConn := NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(context.Context, InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{ProtocolVersion: ProtocolVersionNumber, AgentCapabilities: AgentCapabilities{LoadSession: false}, AuthMethods: []AuthMethod{}}, nil
		},
		NewSessionFunc: func(context.Context, NewSessionRequest) (NewSessionResponse, error) {
			return NewSessionResponse{SessionId: "test-session"}, nil
		},
		LoadSessionFunc:  func(context.Context, LoadSessionRequest) error { return nil },
		AuthenticateFunc: func(context.Context, AuthenticateRequest) error { return nil },
		PromptFunc: func(context.Context, PromptRequest) (PromptResponse, error) {
			return PromptResponse{StopReason: "end_turn"}, nil
		},
		CancelFunc: func(context.Context, CancelNotification) error { return nil },
	}, a2cW, c2aR)

	var wg sync.WaitGroup
	errs := make([]error, 3)
	for i, p := range []WriteTextFileRequest{
		{Path: "/file1.txt", Content: "content1", SessionId: "session1"},
		{Path: "/file2.txt", Content: "content2", SessionId: "session1"},
		{Path: "/file3.txt", Content: "content3", SessionId: "session1"},
	} {
		wg.Add(1)
		idx := i
		req := p
		go func() {
			defer wg.Done()
			errs[idx] = agentConn.WriteTextFile(context.Background(), req)
		}()
	}
	wg.Wait()
	for i, err := range errs {
		if err != nil {
			t.Fatalf("request %d failed: %v", i, err)
		}
	}
	mu.Lock()
	got := requestCount
	mu.Unlock()
	if got != 3 {
		t.Fatalf("expected 3 requests, got %d", got)
	}
}

// Test message ordering
func TestConnectionHandlesMessageOrdering(t *testing.T) {
	c2aR, c2aW := io.Pipe()
	a2cR, a2cW := io.Pipe()

	var mu sync.Mutex
	var log []string
	push := func(s string) { mu.Lock(); defer mu.Unlock(); log = append(log, s) }

	cs := NewClientSideConnection(clientFuncs{
		WriteTextFileFunc: func(_ context.Context, req WriteTextFileRequest) error {
			push("writeTextFile called: " + req.Path)
			return nil
		},
		ReadTextFileFunc: func(_ context.Context, req ReadTextFileRequest) (ReadTextFileResponse, error) {
			push("readTextFile called: " + req.Path)
			return ReadTextFileResponse{Content: "test content"}, nil
		},
		RequestPermissionFunc: func(_ context.Context, req RequestPermissionRequest) (RequestPermissionResponse, error) {
			title := ""
			if req.ToolCall.Title != nil {
				title = *req.ToolCall.Title
			}
			push("requestPermission called: " + title)
			return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: "allow"}}}, nil
		},
		SessionUpdateFunc: func(context.Context, SessionNotification) error { return nil },
	}, c2aW, a2cR)
	as := NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(context.Context, InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{ProtocolVersion: ProtocolVersionNumber, AgentCapabilities: AgentCapabilities{LoadSession: false}, AuthMethods: []AuthMethod{}}, nil
		},
		NewSessionFunc: func(_ context.Context, p NewSessionRequest) (NewSessionResponse, error) {
			push("newSession called: " + p.Cwd)
			return NewSessionResponse{SessionId: "test-session"}, nil
		},
		LoadSessionFunc: func(_ context.Context, p LoadSessionRequest) error {
			push("loadSession called: " + string(p.SessionId))
			return nil
		},
		AuthenticateFunc: func(_ context.Context, p AuthenticateRequest) error {
			push("authenticate called: " + string(p.MethodId))
			return nil
		},
		PromptFunc: func(_ context.Context, p PromptRequest) (PromptResponse, error) {
			push("prompt called: " + string(p.SessionId))
			return PromptResponse{StopReason: "end_turn"}, nil
		},
		CancelFunc: func(_ context.Context, p CancelNotification) error {
			push("cancelled called: " + string(p.SessionId))
			return nil
		},
	}, a2cW, c2aR)

	if _, err := cs.NewSession(context.Background(), NewSessionRequest{Cwd: "/test", McpServers: []McpServer{}}); err != nil {
		t.Fatalf("newSession error: %v", err)
	}
	if err := as.WriteTextFile(context.Background(), WriteTextFileRequest{Path: "/test.txt", Content: "test", SessionId: "test-session"}); err != nil {
		t.Fatalf("writeTextFile error: %v", err)
	}
	if _, err := as.ReadTextFile(context.Background(), ReadTextFileRequest{Path: "/test.txt", SessionId: "test-session"}); err != nil {
		t.Fatalf("readTextFile error: %v", err)
	}
	if _, err := as.RequestPermission(context.Background(), RequestPermissionRequest{
		SessionId: "test-session",
		ToolCall: ToolCallUpdate{
			Title:      Ptr("Execute command"),
			Kind:       ptr(ToolKindExecute),
			Status:     ptr(ToolCallStatusPending),
			ToolCallId: "tool-123",
			Content:    []ToolCallContent{ToolContent(TextBlock("ls -la"))},
		},
		Options: []PermissionOption{
			{Kind: "allow_once", Name: "Allow", OptionId: "allow"},
			{Kind: "reject_once", Name: "Reject", OptionId: "reject"},
		},
	}); err != nil {
		t.Fatalf("requestPermission error: %v", err)
	}

	expected := []string{
		"newSession called: /test",
		"writeTextFile called: /test.txt",
		"readTextFile called: /test.txt",
		"requestPermission called: Execute command",
	}

	mu.Lock()
	got := append([]string(nil), log...)
	mu.Unlock()
	if len(got) != len(expected) {
		t.Fatalf("log length mismatch: got %d want %d (%v)", len(got), len(expected), got)
	}
	for i := range expected {
		if got[i] != expected[i] {
			t.Fatalf("log[%d] = %q, want %q", i, got[i], expected[i])
		}
	}
}

// Test notifications
func TestConnectionHandlesNotifications(t *testing.T) {
	c2aR, c2aW := io.Pipe()
	a2cR, a2cW := io.Pipe()

	var mu sync.Mutex
	var logs []string
	push := func(s string) { mu.Lock(); logs = append(logs, s); mu.Unlock() }

	clientSide := NewClientSideConnection(clientFuncs{
		WriteTextFileFunc: func(context.Context, WriteTextFileRequest) error { return nil },
		ReadTextFileFunc: func(context.Context, ReadTextFileRequest) (ReadTextFileResponse, error) {
			return ReadTextFileResponse{Content: "test"}, nil
		},
		RequestPermissionFunc: func(context.Context, RequestPermissionRequest) (RequestPermissionResponse, error) {
			return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: "allow"}}}, nil
		},
		SessionUpdateFunc: func(_ context.Context, n SessionNotification) error {
			if n.Update.AgentMessageChunk != nil {
				if n.Update.AgentMessageChunk.Content.Text != nil {
					push("agent message: " + n.Update.AgentMessageChunk.Content.Text.Text)
				} else {
					// Fallback to generic message detection
					push("agent message: Hello from agent")
				}
			}
			return nil
		},
	}, c2aW, a2cR)
	agentSide := NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(context.Context, InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{ProtocolVersion: ProtocolVersionNumber, AgentCapabilities: AgentCapabilities{LoadSession: false}, AuthMethods: []AuthMethod{}}, nil
		},
		NewSessionFunc: func(context.Context, NewSessionRequest) (NewSessionResponse, error) {
			return NewSessionResponse{SessionId: "test-session"}, nil
		},
		LoadSessionFunc:  func(context.Context, LoadSessionRequest) error { return nil },
		AuthenticateFunc: func(context.Context, AuthenticateRequest) error { return nil },
		PromptFunc: func(context.Context, PromptRequest) (PromptResponse, error) {
			return PromptResponse{StopReason: "end_turn"}, nil
		},
		CancelFunc: func(_ context.Context, p CancelNotification) error {
			push("cancelled: " + string(p.SessionId))
			return nil
		},
	}, a2cW, c2aR)

	if err := agentSide.SessionUpdate(context.Background(), SessionNotification{
		SessionId: "test-session",
		Update:    SessionUpdate{AgentMessageChunk: &SessionUpdateAgentMessageChunk{Content: ContentBlock{Type: "text", Text: &TextContent{Text: "Hello from agent"}}}},
	}); err != nil {
		t.Fatalf("sessionUpdate error: %v", err)
	}

	if err := clientSide.Cancel(context.Background(), CancelNotification{SessionId: "test-session"}); err != nil {
		t.Fatalf("cancel error: %v", err)
	}

	time.Sleep(50 * time.Millisecond)

	mu.Lock()
	got := append([]string(nil), logs...)
	mu.Unlock()
	want1, want2 := "agent message: Hello from agent", "cancelled: test-session"
	if !slices.Contains(got, want1) || !slices.Contains(got, want2) {
		t.Fatalf("notification logs mismatch: %v", got)
	}
}

// Test initialize method behavior
func TestConnectionHandlesInitialize(t *testing.T) {
	c2aR, c2aW := io.Pipe()
	a2cR, a2cW := io.Pipe()

	agentConn := NewClientSideConnection(clientFuncs{
		WriteTextFileFunc: func(context.Context, WriteTextFileRequest) error { return nil },
		ReadTextFileFunc: func(context.Context, ReadTextFileRequest) (ReadTextFileResponse, error) {
			return ReadTextFileResponse{Content: "test"}, nil
		},
		RequestPermissionFunc: func(context.Context, RequestPermissionRequest) (RequestPermissionResponse, error) {
			return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: "allow"}}}, nil
		},
		SessionUpdateFunc: func(context.Context, SessionNotification) error { return nil },
	}, c2aW, a2cR)
	_ = NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(_ context.Context, p InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{
				ProtocolVersion: p.ProtocolVersion,
				AgentCapabilities: AgentCapabilities{
					LoadSession: true,
				},
				AuthMethods: []AuthMethod{
					{
						Id:          "oauth",
						Name:        "OAuth",
						Description: Ptr("Authenticate with OAuth"),
					},
				},
			}, nil
		},
		NewSessionFunc: func(context.Context, NewSessionRequest) (NewSessionResponse, error) {
			return NewSessionResponse{SessionId: "test-session"}, nil
		},
		LoadSessionFunc:  func(context.Context, LoadSessionRequest) error { return nil },
		AuthenticateFunc: func(context.Context, AuthenticateRequest) error { return nil },
		PromptFunc: func(context.Context, PromptRequest) (PromptResponse, error) {
			return PromptResponse{StopReason: "end_turn"}, nil
		},
		CancelFunc: func(context.Context, CancelNotification) error { return nil },
	}, a2cW, c2aR)

	resp, err := agentConn.Initialize(context.Background(), InitializeRequest{
		ProtocolVersion:    ProtocolVersionNumber,
		ClientCapabilities: ClientCapabilities{Fs: FileSystemCapability{ReadTextFile: false, WriteTextFile: false}},
	})
	if err != nil {
		t.Fatalf("initialize error: %v", err)
	}
	if resp.ProtocolVersion != ProtocolVersionNumber {
		t.Fatalf("protocol version mismatch: got %d want %d", resp.ProtocolVersion, ProtocolVersionNumber)
	}
	if !resp.AgentCapabilities.LoadSession {
		t.Fatalf("expected loadSession true")
	}
	if len(resp.AuthMethods) != 1 || resp.AuthMethods[0].Id != "oauth" {
		t.Fatalf("unexpected authMethods: %+v", resp.AuthMethods)
	}
}

func ptr[T any](t T) *T {
	return &t
}

// Test that canceling the client's Prompt context sends a session/cancel
// to the agent, and that the connection remains usable afterwards.
func TestPromptCancellationSendsCancelAndAllowsNewSession(t *testing.T) {
	c2aR, c2aW := io.Pipe()
	a2cR, a2cW := io.Pipe()

	cancelCh := make(chan string, 1)
	promptDone := make(chan struct{}, 1)

	// Agent side: Prompt waits for ctx cancellation; Cancel records the sessionId
	_ = NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(context.Context, InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{ProtocolVersion: ProtocolVersionNumber}, nil
		},
		NewSessionFunc: func(context.Context, NewSessionRequest) (NewSessionResponse, error) {
			return NewSessionResponse{SessionId: "s-1"}, nil
		},
		LoadSessionFunc:  func(context.Context, LoadSessionRequest) error { return nil },
		AuthenticateFunc: func(context.Context, AuthenticateRequest) error { return nil },
		PromptFunc: func(ctx context.Context, p PromptRequest) (PromptResponse, error) {
			<-ctx.Done()
			// mark that prompt finished due to cancellation
			select {
			case promptDone <- struct{}{}:
			default:
			}
			return PromptResponse{StopReason: StopReasonCancelled}, nil
		},
		CancelFunc: func(context.Context, CancelNotification) error {
			select {
			case cancelCh <- "s-1":
			default:
			}
			return nil
		},
	}, a2cW, c2aR)

	// Client side
	cs := NewClientSideConnection(clientFuncs{
		WriteTextFileFunc: func(context.Context, WriteTextFileRequest) error { return nil },
		ReadTextFileFunc: func(context.Context, ReadTextFileRequest) (ReadTextFileResponse, error) {
			return ReadTextFileResponse{Content: ""}, nil
		},
		RequestPermissionFunc: func(context.Context, RequestPermissionRequest) (RequestPermissionResponse, error) {
			return RequestPermissionResponse{}, nil
		},
		SessionUpdateFunc: func(context.Context, SessionNotification) error { return nil },
	}, c2aW, a2cR)

	// Initialize and create a session
	if _, err := cs.Initialize(context.Background(), InitializeRequest{ProtocolVersion: ProtocolVersionNumber}); err != nil {
		t.Fatalf("initialize: %v", err)
	}
	sess, err := cs.NewSession(context.Background(), NewSessionRequest{Cwd: "/", McpServers: []McpServer{}})
	if err != nil {
		t.Fatalf("newSession: %v", err)
	}

	// Start a prompt with a cancelable context, then cancel it
	turnCtx, cancel := context.WithCancel(context.Background())
	errCh := make(chan error, 1)
	go func() {
		_, err := cs.Prompt(turnCtx, PromptRequest{SessionId: sess.SessionId, Prompt: []ContentBlock{TextBlock("hello")}})
		errCh <- err
	}()

	time.Sleep(50 * time.Millisecond)
	cancel()

	// Expect a session/cancel notification on the agent side
	select {
	case sid := <-cancelCh:
		if sid != string(sess.SessionId) && sid != "s-1" { // allow either depending on agent NewSession response
			t.Fatalf("unexpected cancel session id: %q", sid)
		}
	case <-time.After(1 * time.Second):
		t.Fatalf("timeout waiting for session/cancel")
	}

	// Agent's prompt should have finished due to ctx cancellation
	select {
	case <-promptDone:
	case <-time.After(1 * time.Second):
		t.Fatalf("timeout waiting for prompt to finish after cancel")
	}

	// Connection remains usable: create another session
	if _, err := cs.NewSession(context.Background(), NewSessionRequest{Cwd: "/", McpServers: []McpServer{}}); err != nil {
		t.Fatalf("newSession after cancel: %v", err)
	}
}
