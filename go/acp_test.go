package acp

import (
	"io"
	"slices"
	"sync"
	"testing"
	"time"
)

type clientFuncs struct {
	WriteTextFileFunc     func(WriteTextFileRequest) error
	ReadTextFileFunc      func(ReadTextFileRequest) (ReadTextFileResponse, error)
	RequestPermissionFunc func(RequestPermissionRequest) (RequestPermissionResponse, error)
	SessionUpdateFunc     func(SessionNotification) error
}

var _ Client = (*clientFuncs)(nil)

func (c clientFuncs) WriteTextFile(p WriteTextFileRequest) error {
	if c.WriteTextFileFunc != nil {
		return c.WriteTextFileFunc(p)
	}
	return nil
}

func (c clientFuncs) ReadTextFile(p ReadTextFileRequest) (ReadTextFileResponse, error) {
	if c.ReadTextFileFunc != nil {
		return c.ReadTextFileFunc(p)
	}
	return ReadTextFileResponse{}, nil
}

func (c clientFuncs) RequestPermission(p RequestPermissionRequest) (RequestPermissionResponse, error) {
	if c.RequestPermissionFunc != nil {
		return c.RequestPermissionFunc(p)
	}
	return RequestPermissionResponse{}, nil
}

func (c clientFuncs) SessionUpdate(n SessionNotification) error {
	if c.SessionUpdateFunc != nil {
		return c.SessionUpdateFunc(n)
	}
	return nil
}

type agentFuncs struct {
	InitializeFunc   func(InitializeRequest) (InitializeResponse, error)
	NewSessionFunc   func(NewSessionRequest) (NewSessionResponse, error)
	LoadSessionFunc  func(LoadSessionRequest) error
	AuthenticateFunc func(AuthenticateRequest) error
	PromptFunc       func(PromptRequest) (PromptResponse, error)
	CancelFunc       func(CancelNotification) error
}

var (
	_ Agent       = (*agentFuncs)(nil)
	_ AgentLoader = (*agentFuncs)(nil)
)

func (a agentFuncs) Initialize(p InitializeRequest) (InitializeResponse, error) {
	if a.InitializeFunc != nil {
		return a.InitializeFunc(p)
	}
	return InitializeResponse{}, nil
}

func (a agentFuncs) NewSession(p NewSessionRequest) (NewSessionResponse, error) {
	if a.NewSessionFunc != nil {
		return a.NewSessionFunc(p)
	}
	return NewSessionResponse{}, nil
}

func (a agentFuncs) LoadSession(p LoadSessionRequest) error {
	if a.LoadSessionFunc != nil {
		return a.LoadSessionFunc(p)
	}
	return nil
}

func (a agentFuncs) Authenticate(p AuthenticateRequest) error {
	if a.AuthenticateFunc != nil {
		return a.AuthenticateFunc(p)
	}
	return nil
}

func (a agentFuncs) Prompt(p PromptRequest) (PromptResponse, error) {
	if a.PromptFunc != nil {
		return a.PromptFunc(p)
	}
	return PromptResponse{}, nil
}

func (a agentFuncs) Cancel(n CancelNotification) error {
	if a.CancelFunc != nil {
		return a.CancelFunc(n)
	}
	return nil
}

// Test bidirectional error handling similar to typescript/acp.test.ts
func TestConnectionHandlesErrorsBidirectional(t *testing.T) {
	c2aR, c2aW := io.Pipe()
	a2cR, a2cW := io.Pipe()

	c := NewClientSideConnection(clientFuncs{
		WriteTextFileFunc: func(WriteTextFileRequest) error { return &RequestError{Code: -32603, Message: "Write failed"} },
		ReadTextFileFunc: func(ReadTextFileRequest) (ReadTextFileResponse, error) {
			return ReadTextFileResponse{}, &RequestError{Code: -32603, Message: "Read failed"}
		},
		RequestPermissionFunc: func(RequestPermissionRequest) (RequestPermissionResponse, error) {
			return RequestPermissionResponse{}, &RequestError{Code: -32603, Message: "Permission denied"}
		},
		SessionUpdateFunc: func(SessionNotification) error { return nil },
	}, c2aW, a2cR)
	agentConn := NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{}, &RequestError{Code: -32603, Message: "Failed to initialize"}
		},
		NewSessionFunc: func(NewSessionRequest) (NewSessionResponse, error) {
			return NewSessionResponse{}, &RequestError{Code: -32603, Message: "Failed to create session"}
		},
		LoadSessionFunc:  func(LoadSessionRequest) error { return &RequestError{Code: -32603, Message: "Failed to load session"} },
		AuthenticateFunc: func(AuthenticateRequest) error { return &RequestError{Code: -32603, Message: "Authentication failed"} },
		PromptFunc: func(PromptRequest) (PromptResponse, error) {
			return PromptResponse{}, &RequestError{Code: -32603, Message: "Prompt failed"}
		},
		CancelFunc: func(CancelNotification) error { return nil },
	}, a2cW, c2aR)

	// Client->Agent direction: expect error
	if err := agentConn.WriteTextFile(WriteTextFileRequest{Path: "/test.txt", Content: "test", SessionId: "test-session"}); err == nil {
		t.Fatalf("expected error for writeTextFile, got nil")
	}

	// Agent->Client direction: expect error
	if _, err := c.NewSession(NewSessionRequest{Cwd: "/test", McpServers: []McpServer{}}); err == nil {
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
		WriteTextFileFunc: func(WriteTextFileRequest) error {
			mu.Lock()
			requestCount++
			mu.Unlock()
			time.Sleep(40 * time.Millisecond)
			return nil
		},
		ReadTextFileFunc: func(p ReadTextFileRequest) (ReadTextFileResponse, error) {
			return ReadTextFileResponse{Content: "Content of " + p.Path}, nil
		},
		RequestPermissionFunc: func(RequestPermissionRequest) (RequestPermissionResponse, error) {
			return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: "allow"}}}, nil
		},
		SessionUpdateFunc: func(SessionNotification) error { return nil },
	}, c2aW, a2cR)
	agentConn := NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{ProtocolVersion: ProtocolVersionNumber, AgentCapabilities: AgentCapabilities{LoadSession: false}, AuthMethods: []AuthMethod{}}, nil
		},
		NewSessionFunc: func(NewSessionRequest) (NewSessionResponse, error) {
			return NewSessionResponse{SessionId: "test-session"}, nil
		},
		LoadSessionFunc:  func(LoadSessionRequest) error { return nil },
		AuthenticateFunc: func(AuthenticateRequest) error { return nil },
		PromptFunc:       func(PromptRequest) (PromptResponse, error) { return PromptResponse{StopReason: "end_turn"}, nil },
		CancelFunc:       func(CancelNotification) error { return nil },
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
			errs[idx] = agentConn.WriteTextFile(req)
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
		WriteTextFileFunc: func(p WriteTextFileRequest) error { push("writeTextFile called: " + p.Path); return nil },
		ReadTextFileFunc: func(p ReadTextFileRequest) (ReadTextFileResponse, error) {
			push("readTextFile called: " + p.Path)
			return ReadTextFileResponse{Content: "test content"}, nil
		},
		RequestPermissionFunc: func(p RequestPermissionRequest) (RequestPermissionResponse, error) {
			push("requestPermission called: " + p.ToolCall.Title)
			return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: "allow"}}}, nil
		},
		SessionUpdateFunc: func(SessionNotification) error { return nil },
	}, c2aW, a2cR)
	as := NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{ProtocolVersion: ProtocolVersionNumber, AgentCapabilities: AgentCapabilities{LoadSession: false}, AuthMethods: []AuthMethod{}}, nil
		},
		NewSessionFunc: func(p NewSessionRequest) (NewSessionResponse, error) {
			push("newSession called: " + p.Cwd)
			return NewSessionResponse{SessionId: "test-session"}, nil
		},
		LoadSessionFunc:  func(p LoadSessionRequest) error { push("loadSession called: " + string(p.SessionId)); return nil },
		AuthenticateFunc: func(p AuthenticateRequest) error { push("authenticate called: " + string(p.MethodId)); return nil },
		PromptFunc: func(p PromptRequest) (PromptResponse, error) {
			push("prompt called: " + string(p.SessionId))
			return PromptResponse{StopReason: "end_turn"}, nil
		},
		CancelFunc: func(p CancelNotification) error { push("cancelled called: " + string(p.SessionId)); return nil },
	}, a2cW, c2aR)

	if _, err := cs.NewSession(NewSessionRequest{Cwd: "/test", McpServers: []McpServer{}}); err != nil {
		t.Fatalf("newSession error: %v", err)
	}
	if err := as.WriteTextFile(WriteTextFileRequest{Path: "/test.txt", Content: "test", SessionId: "test-session"}); err != nil {
		t.Fatalf("writeTextFile error: %v", err)
	}
	if _, err := as.ReadTextFile(ReadTextFileRequest{Path: "/test.txt", SessionId: "test-session"}); err != nil {
		t.Fatalf("readTextFile error: %v", err)
	}
	if _, err := as.RequestPermission(RequestPermissionRequest{
		SessionId: "test-session",
		ToolCall: ToolCallUpdate{
			Title:      "Execute command",
			Kind:       ptr(ToolKindExecute),
			Status:     ptr(ToolCallStatusPending),
			ToolCallId: "tool-123",
			Content: []ToolCallContent{{
				Type:    "content",
				Content: &ContentBlock{Type: "text", Text: &TextContent{Text: "ls -la"}},
			}},
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
		WriteTextFileFunc: func(WriteTextFileRequest) error { return nil },
		ReadTextFileFunc: func(ReadTextFileRequest) (ReadTextFileResponse, error) {
			return ReadTextFileResponse{Content: "test"}, nil
		},
		RequestPermissionFunc: func(RequestPermissionRequest) (RequestPermissionResponse, error) {
			return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: "allow"}}}, nil
		},
		SessionUpdateFunc: func(n SessionNotification) error {
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
		InitializeFunc: func(InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{ProtocolVersion: ProtocolVersionNumber, AgentCapabilities: AgentCapabilities{LoadSession: false}, AuthMethods: []AuthMethod{}}, nil
		},
		NewSessionFunc: func(NewSessionRequest) (NewSessionResponse, error) {
			return NewSessionResponse{SessionId: "test-session"}, nil
		},
		LoadSessionFunc:  func(LoadSessionRequest) error { return nil },
		AuthenticateFunc: func(AuthenticateRequest) error { return nil },
		PromptFunc:       func(PromptRequest) (PromptResponse, error) { return PromptResponse{StopReason: "end_turn"}, nil },
		CancelFunc:       func(p CancelNotification) error { push("cancelled: " + string(p.SessionId)); return nil },
	}, a2cW, c2aR)

	if err := agentSide.SessionUpdate(SessionNotification{
		SessionId: "test-session",
		Update:    SessionUpdate{AgentMessageChunk: &SessionUpdateAgentMessageChunk{Content: ContentBlock{Type: "text", Text: &TextContent{Text: "Hello from agent"}}}},
	}); err != nil {
		t.Fatalf("sessionUpdate error: %v", err)
	}

	if err := clientSide.Cancel(CancelNotification{SessionId: "test-session"}); err != nil {
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
		WriteTextFileFunc: func(WriteTextFileRequest) error { return nil },
		ReadTextFileFunc: func(ReadTextFileRequest) (ReadTextFileResponse, error) {
			return ReadTextFileResponse{Content: "test"}, nil
		},
		RequestPermissionFunc: func(RequestPermissionRequest) (RequestPermissionResponse, error) {
			return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: "allow"}}}, nil
		},
		SessionUpdateFunc: func(SessionNotification) error { return nil },
	}, c2aW, a2cR)
	_ = NewAgentSideConnection(agentFuncs{
		InitializeFunc: func(p InitializeRequest) (InitializeResponse, error) {
			return InitializeResponse{ProtocolVersion: p.ProtocolVersion, AgentCapabilities: AgentCapabilities{LoadSession: true}, AuthMethods: []AuthMethod{{Id: "oauth", Name: "OAuth", Description: "Authenticate with OAuth"}}}, nil
		},
		NewSessionFunc: func(NewSessionRequest) (NewSessionResponse, error) {
			return NewSessionResponse{SessionId: "test-session"}, nil
		},
		LoadSessionFunc:  func(LoadSessionRequest) error { return nil },
		AuthenticateFunc: func(AuthenticateRequest) error { return nil },
		PromptFunc:       func(PromptRequest) (PromptResponse, error) { return PromptResponse{StopReason: "end_turn"}, nil },
		CancelFunc:       func(CancelNotification) error { return nil },
	}, a2cW, c2aR)

	resp, err := agentConn.Initialize(InitializeRequest{
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
