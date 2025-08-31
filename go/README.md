<a href="https://agentclientprotocol.com/" >
  <img alt="Agent Client Protocol" src="https://zed.dev/img/acp/banner-dark.webp">
</a>

# ACP Go Library

The official Go implementation of the Agent Client Protocol (ACP) — a standardized communication protocol between code editors and AI‑powered coding agents.

Learn more at <https://agentclientprotocol.com>

## Installation

```bash
go get github.com/zed-industries/agent-client-protocol/go@latest
```

## Get Started

### Understand the Protocol

Start by reading the [official ACP documentation](https://agentclientprotocol.com) to understand the core concepts and protocol specification.

### Try the Examples

The [examples directory](https://github.com/zed-industries/agent-client-protocol/tree/main/go/example) contains simple implementations of both Agents and Clients in Go. You can run them from your terminal or connect to external ACP agents.

- Run the example Agent:
  - `cd go && go run ./example/agent`
- Run the example Client (connects to the example Agent):
  - `cd go && go run ./example/client`
- Connect to the Gemini CLI (ACP mode):
  - `cd go && go run ./example/gemini -yolo`
  - Optional flags: `-model`, `-sandbox`, `-debug`, `-gemini /path/to/gemini`
- Connect to Claude Code (via npx):
  - `cd go && go run ./example/claude-code -yolo`

### Explore the API

Browse the Go package docs on pkg.go.dev for detailed API documentation:

- <https://pkg.go.dev/github.com/zed-industries/agent-client-protocol/go>

If you're building an [Agent](https://agentclientprotocol.com/protocol/overview#agent):

- Implement the `acp.Agent` interface (and optionally `acp.AgentLoader` for `session/load`).
- Create a connection with `acp.NewAgentSideConnection(agent, os.Stdout, os.Stdin)`.
- Send updates and make client requests using the returned connection.

If you're building a [Client](https://agentclientprotocol.com/protocol/overview#client):

- Implement the `acp.Client` interface (and optionally `acp.ClientTerminal` for terminal features).
- Launch or connect to your Agent process (stdio), then create a connection with `acp.NewClientSideConnection(client, stdin, stdout)`.
- Call `Initialize`, `NewSession`, and `Prompt` to run a turn and stream updates.

Helper constructors are provided to reduce boilerplate when working with union types:

- Content blocks: `acp.TextBlock`, `acp.ImageBlock`, `acp.AudioBlock`, `acp.ResourceLinkBlock`, `acp.ResourceBlock`.
- Tool content: `acp.ToolContent`, `acp.ToolDiffContent`, `acp.ToolTerminalRef`.
- Utility: `acp.Ptr[T]` for pointer fields in request/update structs.

### Study a Production Implementation

For a complete, production‑ready integration, see the [Gemini CLI Agent](https://github.com/google-gemini/gemini-cli) which exposes an ACP interface. The Go example client `go/example/gemini` demonstrates connecting to it via stdio.

## Resources

- [Go package docs](https://pkg.go.dev/github.com/zed-industries/agent-client-protocol/go)
- [Examples (Go)](https://github.com/zed-industries/agent-client-protocol/tree/main/go/example)
- [Protocol Documentation](https://agentclientprotocol.com)
- [GitHub Repository](https://github.com/zed-industries/agent-client-protocol)

## Contributing

See the main [repository](https://github.com/zed-industries/agent-client-protocol) for contribution guidelines.
