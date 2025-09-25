package acp

import (
	"context"
	"io"
	"log/slog"
	"sync"
)

// AgentSideConnection represents the agent's view of a connection to a client.
type AgentSideConnection struct {
	conn  *Connection
	agent Agent

	mu             sync.Mutex
	sessionCancels map[string]context.CancelFunc
}

// NewAgentSideConnection creates a new agent-side connection bound to the
// provided Agent implementation.
func NewAgentSideConnection(agent Agent, peerInput io.Writer, peerOutput io.Reader) *AgentSideConnection {
	asc := &AgentSideConnection{}
	asc.agent = agent
	asc.sessionCancels = make(map[string]context.CancelFunc)
	asc.conn = NewConnection(asc.handle, peerInput, peerOutput)
	return asc
}

// Done exposes a channel that closes when the peer disconnects.
func (c *AgentSideConnection) Done() <-chan struct{} { return c.conn.Done() }

// SetLogger directs connection diagnostics to the provided logger.
func (c *AgentSideConnection) SetLogger(l *slog.Logger) { c.conn.SetLogger(l) }
