package acp

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

// clientExample mirrors go/example/client in a compact form: prints
// streamed updates, handles simple file ops, and picks the first
// permission option.
type clientExample struct{}

func (clientExample) RequestPermission(ctx context.Context, p RequestPermissionRequest) (RequestPermissionResponse, error) {
	if len(p.Options) == 0 {
		return RequestPermissionResponse{
			Outcome: RequestPermissionOutcome{
				Cancelled: &RequestPermissionOutcomeCancelled{},
			},
		}, nil
	}
	return RequestPermissionResponse{
		Outcome: RequestPermissionOutcome{
			Selected: &RequestPermissionOutcomeSelected{OptionId: p.Options[0].OptionId},
		},
	}, nil
}

func (clientExample) SessionUpdate(ctx context.Context, n SessionNotification) error {
	u := n.Update
	switch {
	case u.AgentMessageChunk != nil:
		c := u.AgentMessageChunk.Content
		if c.Text != nil {
			fmt.Print(c.Text.Text)
		}
	case u.ToolCall != nil:
		title := u.ToolCall.Title
		fmt.Printf("\n[tool] %s (%s)\n", title, u.ToolCall.Status)
	case u.ToolCallUpdate != nil:
		fmt.Printf("\n[tool] %s -> %v\n", u.ToolCallUpdate.ToolCallId, u.ToolCallUpdate.Status)
	}
	return nil
}

func (clientExample) WriteTextFile(ctx context.Context, p WriteTextFileRequest) error {
	if !filepath.IsAbs(p.Path) {
		return fmt.Errorf("path must be absolute: %s", p.Path)
	}
	if dir := filepath.Dir(p.Path); dir != "" {
		_ = os.MkdirAll(dir, 0o755)
	}
	return os.WriteFile(p.Path, []byte(p.Content), 0o644)
}

func (clientExample) ReadTextFile(ctx context.Context, p ReadTextFileRequest) (ReadTextFileResponse, error) {
	if !filepath.IsAbs(p.Path) {
		return ReadTextFileResponse{}, fmt.Errorf("path must be absolute: %s", p.Path)
	}
	b, err := os.ReadFile(p.Path)
	if err != nil {
		return ReadTextFileResponse{}, err
	}
	content := string(b)
	if p.Line != nil || p.Limit != nil {
		lines := strings.Split(content, "\n")
		start := 0
		if p.Line != nil && *p.Line > 0 {
			if *p.Line-1 > 0 {
				start = *p.Line - 1
			}
			if start > len(lines) {
				start = len(lines)
			}
		}
		end := len(lines)
		if p.Limit != nil && *p.Limit > 0 && start+*p.Limit < end {
			end = start + *p.Limit
		}
		content = strings.Join(lines[start:end], "\n")
	}
	return ReadTextFileResponse{Content: content}, nil
}

// Terminal interface implementations (minimal stubs for examples)
func (clientExample) CreateTerminal(ctx context.Context, p CreateTerminalRequest) (CreateTerminalResponse, error) {
	// Return a dummy terminal id
	return CreateTerminalResponse{TerminalId: "t-1"}, nil
}

func (clientExample) KillTerminalCommand(ctx context.Context, p KillTerminalCommandRequest) error {
	return nil
}

func (clientExample) ReleaseTerminal(ctx context.Context, p ReleaseTerminalRequest) error {
	return nil
}

func (clientExample) TerminalOutput(ctx context.Context, p TerminalOutputRequest) (TerminalOutputResponse, error) {
	// Provide non-empty output to satisfy validation
	return TerminalOutputResponse{Output: "ok", Truncated: false}, nil
}

func (clientExample) WaitForTerminalExit(ctx context.Context, p WaitForTerminalExitRequest) (WaitForTerminalExitResponse, error) {
	return WaitForTerminalExitResponse{}, nil
}

// Example_client launches the Go agent example, negotiates protocol,
// opens a session, and sends a simple prompt.
func Example_client() {
	ctx := context.Background()
	cmd := exec.Command("go", "run", "./example/agent")
	stdin, _ := cmd.StdinPipe()
	stdout, _ := cmd.StdoutPipe()
	_ = cmd.Start()

	conn := NewClientSideConnection(clientExample{}, stdin, stdout)
	_, _ = conn.Initialize(ctx, InitializeRequest{
		ProtocolVersion: ProtocolVersionNumber,
		ClientCapabilities: ClientCapabilities{
			Fs: FileSystemCapability{
				ReadTextFile:  true,
				WriteTextFile: true,
			},
			Terminal: true,
		},
	})
	sess, _ := conn.NewSession(ctx, NewSessionRequest{
		Cwd:        "/",
		McpServers: []McpServer{},
	})
	_, _ = conn.Prompt(ctx, PromptRequest{
		SessionId: sess.SessionId,
		Prompt:    []ContentBlock{TextBlock("Hello, agent!")},
	})

	_ = cmd.Process.Kill()
}
