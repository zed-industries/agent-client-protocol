package acp

import (
	"context"
	"fmt"
	"os/exec"
)

// geminiClient mirrors go/example/gemini in brief: prints text chunks and
// selects the first permission option. File ops are no-ops here.
type geminiClient struct{}

func (geminiClient) RequestPermission(ctx context.Context, p RequestPermissionRequest) (RequestPermissionResponse, error) {
	if len(p.Options) == 0 {
		return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Cancelled: &RequestPermissionOutcomeCancelled{}}}, nil
	}
	return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: p.Options[0].OptionId}}}, nil
}

func (geminiClient) SessionUpdate(ctx context.Context, n SessionNotification) error {
	if n.Update.AgentMessageChunk != nil {
		c := n.Update.AgentMessageChunk.Content
		if c.Text != nil {
			fmt.Print(c.Text.Text)
		}
	}
	return nil
}

func (geminiClient) ReadTextFile(ctx context.Context, _ ReadTextFileRequest) (ReadTextFileResponse, error) {
	return ReadTextFileResponse{}, nil
}
func (geminiClient) WriteTextFile(ctx context.Context, _ WriteTextFileRequest) error { return nil }

// Terminal interface implementations (minimal stubs for examples)
func (geminiClient) CreateTerminal(ctx context.Context, p CreateTerminalRequest) (CreateTerminalResponse, error) {
	return CreateTerminalResponse{TerminalId: "t-1"}, nil
}

func (geminiClient) KillTerminalCommand(ctx context.Context, p KillTerminalCommandRequest) error {
	return nil
}
func (geminiClient) ReleaseTerminal(ctx context.Context, p ReleaseTerminalRequest) error { return nil }
func (geminiClient) TerminalOutput(ctx context.Context, p TerminalOutputRequest) (TerminalOutputResponse, error) {
	return TerminalOutputResponse{Output: "ok", Truncated: false}, nil
}

func (geminiClient) WaitForTerminalExit(ctx context.Context, p WaitForTerminalExitRequest) (WaitForTerminalExitResponse, error) {
	return WaitForTerminalExitResponse{}, nil
}

// Example_gemini connects to a Gemini CLI speaking ACP over stdio,
// then initializes, opens a session, and sends a prompt.
func Example_gemini() {
	ctx := context.Background()
	cmd := exec.Command("gemini", "--experimental-acp")
	stdin, _ := cmd.StdinPipe()
	stdout, _ := cmd.StdoutPipe()
	_ = cmd.Start()

	conn := NewClientSideConnection(geminiClient{}, stdin, stdout)
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
		Prompt:    []ContentBlock{TextBlock("list files")},
	})
}
