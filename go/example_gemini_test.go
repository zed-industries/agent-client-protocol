package acp

import (
	"fmt"
	"os/exec"
)

// geminiClient mirrors go/example/gemini in brief: prints text chunks and
// selects the first permission option. File ops are no-ops here.
type geminiClient struct{}

func (geminiClient) RequestPermission(p RequestPermissionRequest) (RequestPermissionResponse, error) {
	if len(p.Options) == 0 {
		return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Cancelled: &RequestPermissionOutcomeCancelled{}}}, nil
	}
	return RequestPermissionResponse{Outcome: RequestPermissionOutcome{Selected: &RequestPermissionOutcomeSelected{OptionId: p.Options[0].OptionId}}}, nil
}

func (geminiClient) SessionUpdate(n SessionNotification) error {
	if n.Update.AgentMessageChunk != nil {
		c := n.Update.AgentMessageChunk.Content
		if c.Type == "text" && c.Text != nil {
			fmt.Print(c.Text.Text)
		}
	}
	return nil
}

func (geminiClient) ReadTextFile(ReadTextFileRequest) (ReadTextFileResponse, error) {
	return ReadTextFileResponse{}, nil
}
func (geminiClient) WriteTextFile(WriteTextFileRequest) error { return nil }

// Example_gemini connects to a Gemini CLI speaking ACP over stdio,
// then initializes, opens a session, and sends a prompt.
func Example_gemini() {
	cmd := exec.Command("gemini", "--experimental-acp")
	stdin, _ := cmd.StdinPipe()
	stdout, _ := cmd.StdoutPipe()
	_ = cmd.Start()

	conn := NewClientSideConnection(geminiClient{}, stdin, stdout)
	_, _ = conn.Initialize(InitializeRequest{
		ProtocolVersion: ProtocolVersionNumber,
		ClientCapabilities: ClientCapabilities{
			Fs: FileSystemCapability{
				ReadTextFile:  true,
				WriteTextFile: true,
			},
			Terminal: true,
		},
	})
	sess, _ := conn.NewSession(NewSessionRequest{
		Cwd:        "/",
		McpServers: []McpServer{},
	})
	_, _ = conn.Prompt(PromptRequest{
		SessionId: sess.SessionId,
		Prompt:    []ContentBlock{TextBlock("list files")},
	})
}
