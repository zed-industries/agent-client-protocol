package main

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	acp "github.com/zed-industries/agent-client-protocol/go"
)

type exampleClient struct{}

var (
	_ acp.Client         = (*exampleClient)(nil)
	_ acp.ClientTerminal = (*exampleClient)(nil)
)

func (e *exampleClient) RequestPermission(ctx context.Context, params acp.RequestPermissionRequest) (acp.RequestPermissionResponse, error) {
	fmt.Printf("\n🔐 Permission requested: %s\n", params.ToolCall.Title)
	fmt.Println("\nOptions:")
	for i, opt := range params.Options {
		fmt.Printf("   %d. %s (%s)\n", i+1, opt.Name, opt.Kind)
	}
	reader := bufio.NewReader(os.Stdin)
	for {
		fmt.Printf("\nChoose an option: ")
		line, _ := reader.ReadString('\n')
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		idx := -1
		fmt.Sscanf(line, "%d", &idx)
		idx = idx - 1
		if idx >= 0 && idx < len(params.Options) {
			return acp.RequestPermissionResponse{Outcome: acp.RequestPermissionOutcome{Selected: &acp.RequestPermissionOutcomeSelected{OptionId: params.Options[idx].OptionId}}}, nil
		}
		fmt.Println("Invalid option. Please try again.")
	}
}

func (e *exampleClient) SessionUpdate(ctx context.Context, params acp.SessionNotification) error {
	u := params.Update
	switch {
	case u.AgentMessageChunk != nil:
		c := u.AgentMessageChunk.Content
		if c.Type == "text" && c.Text != nil {
			fmt.Println(c.Text.Text)
		} else {
			fmt.Printf("[%s]\n", c.Type)
		}
	case u.ToolCall != nil:
		fmt.Printf("\n🔧 %s (%s)\n", u.ToolCall.Title, u.ToolCall.Status)
	case u.ToolCallUpdate != nil:
		fmt.Printf("\n🔧 Tool call `%s` updated: %v\n\n", u.ToolCallUpdate.ToolCallId, u.ToolCallUpdate.Status)
	case u.Plan != nil || u.AgentThoughtChunk != nil || u.UserMessageChunk != nil:
		// Keep output compact for other updates
		fmt.Println("[", displayUpdateKind(u), "]")
	}
	return nil
}

func displayUpdateKind(u acp.SessionUpdate) string {
	switch {
	case u.UserMessageChunk != nil:
		return "user_message_chunk"
	case u.AgentMessageChunk != nil:
		return "agent_message_chunk"
	case u.AgentThoughtChunk != nil:
		return "agent_thought_chunk"
	case u.ToolCall != nil:
		return "tool_call"
	case u.ToolCallUpdate != nil:
		return "tool_call_update"
	case u.Plan != nil:
		return "plan"
	default:
		return "unknown"
	}
}

func (e *exampleClient) WriteTextFile(ctx context.Context, params acp.WriteTextFileRequest) error {
	if !filepath.IsAbs(params.Path) {
		return fmt.Errorf("path must be absolute: %s", params.Path)
	}
	dir := filepath.Dir(params.Path)
	if dir != "" {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			return fmt.Errorf("mkdir %s: %w", dir, err)
		}
	}
	if err := os.WriteFile(params.Path, []byte(params.Content), 0o644); err != nil {
		return fmt.Errorf("write %s: %w", params.Path, err)
	}
	fmt.Printf("[Client] Wrote %d bytes to %s\n", len(params.Content), params.Path)
	return nil
}

func (e *exampleClient) ReadTextFile(ctx context.Context, params acp.ReadTextFileRequest) (acp.ReadTextFileResponse, error) {
	if !filepath.IsAbs(params.Path) {
		return acp.ReadTextFileResponse{}, fmt.Errorf("path must be absolute: %s", params.Path)
	}
	b, err := os.ReadFile(params.Path)
	if err != nil {
		return acp.ReadTextFileResponse{}, fmt.Errorf("read %s: %w", params.Path, err)
	}
	content := string(b)
	// Apply optional line/limit (1-based line index)
	if params.Line > 0 || params.Limit > 0 {
		lines := strings.Split(content, "\n")
		start := 0
		if params.Line > 0 {
			start = min(max(params.Line-1, 0), len(lines))
		}
		end := len(lines)
		if params.Limit > 0 {
			if start+params.Limit < end {
				end = start + params.Limit
			}
		}
		content = strings.Join(lines[start:end], "\n")
	}
	fmt.Printf("[Client] ReadTextFile: %s (%d bytes)\n", params.Path, len(content))
	return acp.ReadTextFileResponse{Content: content}, nil
}

// Optional/UNSTABLE terminal methods: implement as no-ops for example
func (e *exampleClient) CreateTerminal(ctx context.Context, params acp.CreateTerminalRequest) (acp.CreateTerminalResponse, error) {
	fmt.Printf("[Client] CreateTerminal: %v\n", params)
	return acp.CreateTerminalResponse{TerminalId: "term-1"}, nil
}

func (e *exampleClient) TerminalOutput(ctx context.Context, params acp.TerminalOutputRequest) (acp.TerminalOutputResponse, error) {
	fmt.Printf("[Client] TerminalOutput: %v\n", params)
	return acp.TerminalOutputResponse{Output: "", Truncated: false}, nil
}

func (e *exampleClient) ReleaseTerminal(ctx context.Context, params acp.ReleaseTerminalRequest) error {
	fmt.Printf("[Client] ReleaseTerminal: %v\n", params)
	return nil
}

func (e *exampleClient) WaitForTerminalExit(ctx context.Context, params acp.WaitForTerminalExitRequest) (acp.WaitForTerminalExitResponse, error) {
	fmt.Printf("[Client] WaitForTerminalExit: %v\n", params)
	return acp.WaitForTerminalExitResponse{}, nil
}

func main() {
	// If args provided, treat them as agent program + args. Otherwise run the Go agent example.
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	var cmd *exec.Cmd
	if len(os.Args) > 1 {
		cmd = exec.CommandContext(ctx, os.Args[1], os.Args[2:]...)
	} else {
		// Assumes running from the go/ directory; if not, adjust path accordingly.
		cmd = exec.CommandContext(ctx, "go", "run", "./example/agent")
	}
	cmd.Stderr = os.Stderr
	cmd.Stdout = nil
	cmd.Stdin = nil
	// Set up pipes for stdio
	stdin, _ := cmd.StdinPipe()
	stdout, _ := cmd.StdoutPipe()
	if err := cmd.Start(); err != nil {
		fmt.Fprintf(os.Stderr, "failed to start agent: %v\n", err)
		os.Exit(1)
	}

	client := &exampleClient{}
	conn := acp.NewClientSideConnection(client, stdin, stdout)

	// Initialize
	initResp, err := conn.Initialize(ctx, acp.InitializeRequest{
		ProtocolVersion: acp.ProtocolVersionNumber,
		ClientCapabilities: acp.ClientCapabilities{
			Fs:       acp.FileSystemCapability{ReadTextFile: true, WriteTextFile: true},
			Terminal: true,
		},
	})
	if err != nil {
		if re, ok := err.(*acp.RequestError); ok {
			if b, mErr := json.MarshalIndent(re, "", "  "); mErr == nil {
				fmt.Fprintf(os.Stderr, "[Client] Error: %s\n", string(b))
			} else {
				fmt.Fprintf(os.Stderr, "initialize error (%d): %s\n", re.Code, re.Message)
			}
		} else {
			fmt.Fprintf(os.Stderr, "initialize error: %v\n", err)
		}
		_ = cmd.Process.Kill()
		os.Exit(1)
	}
	fmt.Printf("✅ Connected to agent (protocol v%v)\n", initResp.ProtocolVersion)

	// New session
	newSess, err := conn.NewSession(ctx, acp.NewSessionRequest{Cwd: mustCwd(), McpServers: []acp.McpServer{}})
	if err != nil {
		if re, ok := err.(*acp.RequestError); ok {
			if b, mErr := json.MarshalIndent(re, "", "  "); mErr == nil {
				fmt.Fprintf(os.Stderr, "[Client] Error: %s\n", string(b))
			} else {
				fmt.Fprintf(os.Stderr, "newSession error (%d): %s\n", re.Code, re.Message)
			}
		} else {
			fmt.Fprintf(os.Stderr, "newSession error: %v\n", err)
		}
		_ = cmd.Process.Kill()
		os.Exit(1)
	}
	fmt.Printf("📝 Created session: %s\n", newSess.SessionId)
	fmt.Printf("💬 User: Hello, agent!\n\n")
	fmt.Print(" ")

	// Send prompt
	if _, err := conn.Prompt(ctx, acp.PromptRequest{
		SessionId: newSess.SessionId,
		Prompt:    []acp.ContentBlock{acp.TextBlock("Hello, agent!")},
	}); err != nil {
		if re, ok := err.(*acp.RequestError); ok {
			if b, mErr := json.MarshalIndent(re, "", "  "); mErr == nil {
				fmt.Fprintf(os.Stderr, "[Client] Error: %s\n", string(b))
			} else {
				fmt.Fprintf(os.Stderr, "prompt error (%d): %s\n", re.Code, re.Message)
			}
		} else {
			fmt.Fprintf(os.Stderr, "prompt error: %v\n", err)
		}
	} else {
		fmt.Printf("\n\n✅ Agent completed\n")
	}

	_ = cmd.Process.Kill()
}

func mustCwd() string {
	wd, err := os.Getwd()
	if err != nil {
		return "."
	}
	return wd
}
