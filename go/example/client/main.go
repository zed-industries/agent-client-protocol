package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"strings"

	acp "github.com/zed-industries/agent-client-protocol/go"
)

type exampleClient struct{}

func (e *exampleClient) RequestPermission(params acp.RequestPermissionRequest) (acp.RequestPermissionResponse, error) {
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

func (e *exampleClient) SessionUpdate(params acp.SessionNotification) error {
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

func (e *exampleClient) WriteTextFile(params acp.WriteTextFileRequest) error {
	fmt.Printf("[Client] Write text file called with: %v\n", params)
	return nil
}

func (e *exampleClient) ReadTextFile(params acp.ReadTextFileRequest) (acp.ReadTextFileResponse, error) {
	fmt.Printf("[Client] Read text file called with: %v\n", params)
	return acp.ReadTextFileResponse{Content: "Mock file content"}, nil
}

func main() {
	// If args provided, treat them as agent program + args. Otherwise run the Go agent example.
	var cmd *exec.Cmd
	if len(os.Args) > 1 {
		cmd = exec.Command(os.Args[1], os.Args[2:]...)
	} else {
		// Assumes running from the go/ directory; if not, adjust path accordingly.
		cmd = exec.Command("go", "run", "./example/agent")
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
	initResp, err := conn.Initialize(acp.InitializeRequest{
		ProtocolVersion:    acp.ProtocolVersionNumber,
		ClientCapabilities: acp.ClientCapabilities{Fs: acp.FileSystemCapability{ReadTextFile: true, WriteTextFile: true}},
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "initialize error: %v\n", err)
		_ = cmd.Process.Kill()
		os.Exit(1)
	}
	fmt.Printf("✅ Connected to agent (protocol v%v)\n", initResp.ProtocolVersion)

	// New session
	newSess, err := conn.NewSession(acp.NewSessionRequest{Cwd: mustCwd(), McpServers: []acp.McpServer{}})
	if err != nil {
		fmt.Fprintf(os.Stderr, "newSession error: %v\n", err)
		_ = cmd.Process.Kill()
		os.Exit(1)
	}
	fmt.Printf("📝 Created session: %s\n", newSess.SessionId)
	fmt.Printf("💬 User: Hello, agent!\n\n")
	fmt.Print(" ")

	// Send prompt
	if _, err := conn.Prompt(acp.PromptRequest{
		SessionId: newSess.SessionId,
		Prompt: []acp.ContentBlock{{
			Type: "text",
			Text: &acp.TextContent{Text: "Hello, agent!"},
		}},
	}); err != nil {
		if re, ok := err.(*acp.RequestError); ok {
			fmt.Fprintf(os.Stderr, "prompt error (%d): %s\n", re.Code, re.Message)
			if re.Data != nil {
				fmt.Fprintf(os.Stderr, "details: %v\n", re.Data)
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
