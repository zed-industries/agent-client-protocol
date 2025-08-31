package main

import (
	"bufio"
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	acp "github.com/zed-industries/agent-client-protocol/go"
)

// ClaudeCodeREPL demonstrates connecting to the Claude Code CLI running in ACP mode
// and providing a simple REPL to send prompts and print streamed updates.

type replClient struct {
	autoApprove bool
}

var _ acp.Client = (*replClient)(nil)

func (c *replClient) RequestPermission(ctx context.Context, params acp.RequestPermissionRequest) (acp.RequestPermissionResponse, error) {
	if c.autoApprove {
		// Prefer an allow option if present; otherwise choose the first option.
		for _, o := range params.Options {
			if o.Kind == acp.PermissionOptionKindAllowOnce || o.Kind == acp.PermissionOptionKindAllowAlways {
				return acp.RequestPermissionResponse{Outcome: acp.RequestPermissionOutcome{Selected: &acp.RequestPermissionOutcomeSelected{OptionId: o.OptionId}}}, nil
			}
		}
		if len(params.Options) > 0 {
			return acp.RequestPermissionResponse{Outcome: acp.RequestPermissionOutcome{Selected: &acp.RequestPermissionOutcomeSelected{OptionId: params.Options[0].OptionId}}}, nil
		}
		return acp.RequestPermissionResponse{Outcome: acp.RequestPermissionOutcome{Cancelled: &acp.RequestPermissionOutcomeCancelled{}}}, nil
	}

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

func (c *replClient) SessionUpdate(ctx context.Context, params acp.SessionNotification) error {
	u := params.Update
	switch {
	case u.AgentMessageChunk != nil:
		content := u.AgentMessageChunk.Content
		if content.Type == "text" && content.Text != nil {
			fmt.Printf("[agent] \n%s\n", content.Text.Text)
		} else {
			fmt.Printf("[agent] %s\n", content.Type)
		}
	case u.ToolCall != nil:
		fmt.Printf("\n🔧 %s (%s)\n", u.ToolCall.Title, u.ToolCall.Status)
	case u.ToolCallUpdate != nil:
		fmt.Printf("\n🔧 Tool call `%s` updated: %v\n\n", u.ToolCallUpdate.ToolCallId, u.ToolCallUpdate.Status)
	case u.Plan != nil:
		fmt.Println("[plan update]")
	case u.AgentThoughtChunk != nil:
		thought := u.AgentThoughtChunk.Content
		if thought.Type == "text" && thought.Text != nil {
			fmt.Printf("[agent_thought_chunk] \n%s\n", thought.Text.Text)
		} else {
			fmt.Println("[agent_thought_chunk]", "(", thought.Type, ")")
		}
	case u.UserMessageChunk != nil:
		fmt.Println("[user_message_chunk]")
	}
	return nil
}

func (c *replClient) WriteTextFile(ctx context.Context, params acp.WriteTextFileRequest) error {
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

func (c *replClient) ReadTextFile(ctx context.Context, params acp.ReadTextFileRequest) (acp.ReadTextFileResponse, error) {
	if !filepath.IsAbs(params.Path) {
		return acp.ReadTextFileResponse{}, fmt.Errorf("path must be absolute: %s", params.Path)
	}
	b, err := os.ReadFile(params.Path)
	if err != nil {
		return acp.ReadTextFileResponse{}, fmt.Errorf("read %s: %w", params.Path, err)
	}
	content := string(b)
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
func (c *replClient) CreateTerminal(ctx context.Context, params acp.CreateTerminalRequest) (acp.CreateTerminalResponse, error) {
	fmt.Printf("[Client] CreateTerminal: %v\n", params)
	return acp.CreateTerminalResponse{TerminalId: "term-1"}, nil
}

func (c *replClient) TerminalOutput(ctx context.Context, params acp.TerminalOutputRequest) (acp.TerminalOutputResponse, error) {
	fmt.Printf("[Client] TerminalOutput: %v\n", params)
	return acp.TerminalOutputResponse{Output: "", Truncated: false}, nil
}

func (c *replClient) ReleaseTerminal(ctx context.Context, params acp.ReleaseTerminalRequest) error {
	fmt.Printf("[Client] ReleaseTerminal: %v\n", params)
	return nil
}

func (c *replClient) WaitForTerminalExit(ctx context.Context, params acp.WaitForTerminalExitRequest) (acp.WaitForTerminalExitResponse, error) {
	fmt.Printf("[Client] WaitForTerminalExit: %v\n", params)
	return acp.WaitForTerminalExitResponse{}, nil
}

func main() {
	yolo := flag.Bool("yolo", false, "Auto-approve permission prompts")
	flag.Parse()

	// Invoke Claude Code via npx
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	cmd := exec.CommandContext(ctx, "npx", "-y", "@zed-industries/claude-code-acp")
	cmd.Stderr = os.Stderr
	stdin, err := cmd.StdinPipe()
	if err != nil {
		fmt.Fprintf(os.Stderr, "stdin pipe error: %v\n", err)
		os.Exit(1)
	}
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		fmt.Fprintf(os.Stderr, "stdout pipe error: %v\n", err)
		os.Exit(1)
	}

	if err := cmd.Start(); err != nil {
		fmt.Fprintf(os.Stderr, "failed to start Claude Code: %v\n", err)
		os.Exit(1)
	}

	client := &replClient{autoApprove: *yolo}
	conn := acp.NewClientSideConnection(client, stdin, stdout)

	// Initialize
	initResp, err := conn.Initialize(ctx, acp.InitializeRequest{
		ProtocolVersion:    acp.ProtocolVersionNumber,
		ClientCapabilities: acp.ClientCapabilities{Fs: acp.FileSystemCapability{ReadTextFile: true, WriteTextFile: true}},
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
	fmt.Printf("✅ Connected to Claude Code (protocol v%v)\n", initResp.ProtocolVersion)

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

	fmt.Println("Type a message and press Enter to send. Commands: :cancel, :exit")
	scanner := bufio.NewScanner(os.Stdin)
	for {
		fmt.Print("> ")
		if !scanner.Scan() {
			break
		}
		line := strings.TrimSpace(scanner.Text())
		if line == "" {
			continue
		}
		switch line {
		case ":exit", ":quit":
			cancel()
			return
		case ":cancel":
			_ = conn.Cancel(ctx, acp.CancelNotification(newSess))
			continue
		}
		// Send prompt and wait for completion while streaming updates are printed via SessionUpdate
		if _, err := conn.Prompt(ctx, acp.PromptRequest{
			SessionId: newSess.SessionId,
			Prompt:    []acp.ContentBlock{acp.TextBlock(line)},
		}); err != nil {
			// If it's a JSON-RPC RequestError, surface more detail for troubleshooting
			if re, ok := err.(*acp.RequestError); ok {
				if b, mErr := json.MarshalIndent(re, "", "  "); mErr == nil {
					fmt.Fprintf(os.Stderr, "[Client] Error: %s\n", string(b))
				} else {
					fmt.Fprintf(os.Stderr, "prompt error (%d): %s\n", re.Code, re.Message)
				}
			} else {
				fmt.Fprintf(os.Stderr, "prompt error: %v\n", err)
			}
		}
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
