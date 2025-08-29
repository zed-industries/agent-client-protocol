package main

import (
	"bufio"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"strings"

	acp "github.com/zed-industries/agent-client-protocol/go"
)

// GeminiREPL demonstrates connecting to the Gemini CLI running in ACP mode
// and providing a simple REPL to send prompts and print streamed updates.

type replClient struct {
	autoApprove bool
}

var _ acp.Client = (*replClient)(nil)

func (c *replClient) RequestPermission(params acp.RequestPermissionRequest) (acp.RequestPermissionResponse, error) {
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

func (c *replClient) SessionUpdate(params acp.SessionNotification) error {
	u := params.Update
	switch {
	case u.AgentMessageChunk != nil:
		content := u.AgentMessageChunk.Content
		if content.Type == "text" && content.Text != nil {
			fmt.Println(content.Text.Text)
		} else {
			fmt.Printf("[%s]\n", content.Type)
		}
	case u.ToolCall != nil:
		fmt.Printf("\n🔧 %s (%s)\n", u.ToolCall.Title, u.ToolCall.Status)
	case u.ToolCallUpdate != nil:
		fmt.Printf("\n🔧 Tool call `%s` updated: %v\n\n", u.ToolCallUpdate.ToolCallId, u.ToolCallUpdate.Status)
	case u.Plan != nil:
		fmt.Println("[plan update]")
	case u.AgentThoughtChunk != nil:
		fmt.Println("[agent_thought_chunk]")
	case u.UserMessageChunk != nil:
		fmt.Println("[user_message_chunk]")
	}
	return nil
}

func (c *replClient) WriteTextFile(params acp.WriteTextFileRequest) error {
	// For demo purposes, just log the request and allow it.
	fmt.Printf("[Client] WriteTextFile: %v\n", params)
	return nil
}

func (c *replClient) ReadTextFile(params acp.ReadTextFileRequest) (acp.ReadTextFileResponse, error) {
	fmt.Printf("[Client] ReadTextFile: %v\n", params)
	return acp.ReadTextFileResponse{Content: "Mock file content"}, nil
}

func main() {
	binary := flag.String("gemini", "gemini", "Path to the Gemini CLI binary")
	model := flag.String("model", "", "Model to pass to Gemini (optional)")
	sandbox := flag.Bool("sandbox", false, "Run Gemini in sandbox mode")
	yolo := flag.Bool("yolo", false, "Auto-approve permission prompts")
	debug := flag.Bool("debug", false, "Pass --debug to Gemini")
	flag.Parse()

	args := []string{"--experimental-acp"}
	if *model != "" {
		args = append(args, "--model", *model)
	}
	if *sandbox {
		args = append(args, "--sandbox")
	}
	if *debug {
		args = append(args, "--debug")
	}

	cmd := exec.Command(*binary, args...)
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
		fmt.Fprintf(os.Stderr, "failed to start Gemini: %v\n", err)
		os.Exit(1)
	}

	client := &replClient{autoApprove: *yolo}
	conn := acp.NewClientSideConnection(client, stdin, stdout)

	// Initialize
	initResp, err := conn.Initialize(acp.InitializeRequest{
		ProtocolVersion:    acp.ProtocolVersionNumber,
		ClientCapabilities: acp.ClientCapabilities{Fs: acp.FileSystemCapability{ReadTextFile: true, WriteTextFile: true}},
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "initialize error: %v\n", err)
		_ = cmd.Process.Kill()
		os.Exit(1)
	}
	fmt.Printf("✅ Connected to Gemini (protocol v%v)\n", initResp.ProtocolVersion)

	// New session
	newSess, err := conn.NewSession(acp.NewSessionRequest{Cwd: mustCwd(), McpServers: []acp.McpServer{}})
	if err != nil {
		fmt.Fprintf(os.Stderr, "newSession error: %v\n", err)
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
			_ = cmd.Process.Kill()
			return
		case ":cancel":
			_ = conn.Cancel(acp.CancelNotification(newSess))
			continue
		}
		// Send prompt and wait for completion while streaming updates are printed via SessionUpdate
		if _, err := conn.Prompt(acp.PromptRequest{
			SessionId: newSess.SessionId,
			Prompt:    []acp.ContentBlock{{Type: "text", Text: &acp.TextContent{Text: line}}},
		}); err != nil {
			// If it's a JSON-RPC RequestError, surface more detail for troubleshooting
			if re, ok := err.(*acp.RequestError); ok {
				fmt.Fprintf(os.Stderr, "prompt error (%d): %s\n", re.Code, re.Message)
				if re.Data != nil {
					fmt.Fprintf(os.Stderr, "details: %v\n", re.Data)
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
