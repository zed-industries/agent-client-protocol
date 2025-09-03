package acp

import (
	"bufio"
	"context"
	"encoding/json"
	"io"
	"sync"
	"sync/atomic"
)

type anyMessage struct {
	JSONRPC string           `json:"jsonrpc"`
	ID      *json.RawMessage `json:"id,omitempty"`
	Method  string           `json:"method,omitempty"`
	Params  json.RawMessage  `json:"params,omitempty"`
	Result  json.RawMessage  `json:"result,omitempty"`
	Error   *RequestError    `json:"error,omitempty"`
}

type pendingResponse struct {
	ch chan anyMessage
}

type MethodHandler func(ctx context.Context, method string, params json.RawMessage) (any, *RequestError)

// Connection is a simple JSON-RPC 2.0 connection over line-delimited JSON.
type Connection struct {
	w       io.Writer
	r       io.Reader
	handler MethodHandler

	mu      sync.Mutex
	nextID  atomic.Uint64
	pending map[string]*pendingResponse

	done chan struct{}
}

func NewConnection(handler MethodHandler, peerInput io.Writer, peerOutput io.Reader) *Connection {
	c := &Connection{
		w:       peerInput,
		r:       peerOutput,
		handler: handler,
		pending: make(map[string]*pendingResponse),
		done:    make(chan struct{}),
	}
	go c.receive()
	return c
}

func (c *Connection) receive() {
	scanner := bufio.NewScanner(c.r)
	// increase buffer if needed
	buf := make([]byte, 0, 1024*1024)
	scanner.Buffer(buf, 10*1024*1024)
	for scanner.Scan() {
		line := scanner.Bytes()
		if len(bytesTrimSpace(line)) == 0 {
			continue
		}
		var msg anyMessage
		if err := json.Unmarshal(line, &msg); err != nil {
			// ignore parse errors on inbound
			continue
		}
		if msg.ID != nil && msg.Method == "" {
			// response
			idStr := string(*msg.ID)
			c.mu.Lock()
			pr := c.pending[idStr]
			if pr != nil {
				delete(c.pending, idStr)
			}
			c.mu.Unlock()
			if pr != nil {
				pr.ch <- msg
			}
			continue
		}
		if msg.Method != "" {
			// request or notification
			go c.handleInbound(&msg)
		}
	}
	// Signal completion on EOF or read error
	c.mu.Lock()
	if c.done != nil {
		close(c.done)
		c.done = nil
	}
	c.mu.Unlock()
}

func (c *Connection) handleInbound(req *anyMessage) {
	// Context that cancels when the connection is closed
	ctx, cancel := context.WithCancel(context.Background())
	go func() {
		<-c.Done()
		cancel()
	}()
	res := anyMessage{JSONRPC: "2.0"}
	// copy ID if present
	if req.ID != nil {
		res.ID = req.ID
	}
	if c.handler == nil {
		if req.ID != nil {
			res.Error = NewMethodNotFound(req.Method)
			_ = c.sendMessage(res)
		}
		return
	}

	result, err := c.handler(ctx, req.Method, req.Params)
	if req.ID == nil {
		// notification: nothing to send
		return
	}
	if err != nil {
		res.Error = err
	} else {
		// marshal result
		b, mErr := json.Marshal(result)
		if mErr != nil {
			res.Error = NewInternalError(map[string]any{"error": mErr.Error()})
		} else {
			res.Result = b
		}
	}
	_ = c.sendMessage(res)
}

func (c *Connection) sendMessage(msg anyMessage) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	msg.JSONRPC = "2.0"
	b, err := json.Marshal(msg)
	if err != nil {
		return err
	}
	b = append(b, '\n')
	_, err = c.w.Write(b)
	return err
}

// SendRequest sends a JSON-RPC request and returns a typed result.
// For methods that do not return a result, use SendRequestNoResult instead.
func SendRequest[T any](c *Connection, ctx context.Context, method string, params any) (T, error) {
	var zero T
	// allocate id
	id := c.nextID.Add(1)
	idRaw, _ := json.Marshal(id)
	msg := anyMessage{
		JSONRPC: "2.0",
		ID:      (*json.RawMessage)(&idRaw),
		Method:  method,
	}
	if params != nil {
		b, err := json.Marshal(params)
		if err != nil {
			return zero, NewInvalidParams(map[string]any{"error": err.Error()})
		}
		msg.Params = b
	}
	pr := &pendingResponse{ch: make(chan anyMessage, 1)}
	idKey := string(idRaw)
	c.mu.Lock()
	c.pending[idKey] = pr
	c.mu.Unlock()
	if err := c.sendMessage(msg); err != nil {
		return zero, NewInternalError(map[string]any{"error": err.Error()})
	}
	// wait for response or peer disconnect
	var resp anyMessage
	d := c.Done()
	select {
	case resp = <-pr.ch:
	case <-ctx.Done():
		// best-effort cleanup
		c.mu.Lock()
		delete(c.pending, idKey)
		c.mu.Unlock()
		return zero, NewInternalError(map[string]any{"error": ctx.Err().Error()})
	case <-d:
		return zero, NewInternalError(map[string]any{"error": "peer disconnected before response"})
	}
	if resp.Error != nil {
		return zero, resp.Error
	}
	var out T
	if len(resp.Result) > 0 {
		if err := json.Unmarshal(resp.Result, &out); err != nil {
			return zero, NewInternalError(map[string]any{"error": err.Error()})
		}
	}
	return out, nil
}

// SendRequestNoResult sends a JSON-RPC request that returns no result payload.
func (c *Connection) SendRequestNoResult(ctx context.Context, method string, params any) error {
	// allocate id
	id := c.nextID.Add(1)
	idRaw, _ := json.Marshal(id)
	msg := anyMessage{
		JSONRPC: "2.0",
		ID:      (*json.RawMessage)(&idRaw),
		Method:  method,
	}
	if params != nil {
		b, err := json.Marshal(params)
		if err != nil {
			return NewInvalidParams(map[string]any{"error": err.Error()})
		}
		msg.Params = b
	}
	pr := &pendingResponse{ch: make(chan anyMessage, 1)}
	idKey := string(idRaw)
	c.mu.Lock()
	c.pending[idKey] = pr
	c.mu.Unlock()
	if err := c.sendMessage(msg); err != nil {
		return NewInternalError(map[string]any{"error": err.Error()})
	}
	var resp anyMessage
	d := c.Done()
	select {
	case resp = <-pr.ch:
	case <-ctx.Done():
		c.mu.Lock()
		delete(c.pending, idKey)
		c.mu.Unlock()
		return NewInternalError(map[string]any{"error": ctx.Err().Error()})
	case <-d:
		return NewInternalError(map[string]any{"error": "peer disconnected before response"})
	}
	if resp.Error != nil {
		return resp.Error
	}
	return nil
}

func (c *Connection) SendNotification(ctx context.Context, method string, params any) error {
	select {
	case <-ctx.Done():
		return NewInternalError(map[string]any{"error": ctx.Err().Error()})
	default:
	}
	msg := anyMessage{JSONRPC: "2.0", Method: method}
	if params != nil {
		b, err := json.Marshal(params)
		if err != nil {
			return NewInvalidParams(map[string]any{"error": err.Error()})
		}
		msg.Params = b
	}
	if err := c.sendMessage(msg); err != nil {
		return NewInternalError(map[string]any{"error": err.Error()})
	}
	return nil
}

// Done returns a channel that is closed when the underlying reader loop exits
// (typically when the peer disconnects or the input stream is closed).
func (c *Connection) Done() <-chan struct{} {
	c.mu.Lock()
	d := c.done
	c.mu.Unlock()
	return d
}

// Helper: lightweight TrimSpace for []byte without importing bytes only for this.
func bytesTrimSpace(b []byte) []byte {
	i := 0
	for ; i < len(b); i++ {
		if b[i] != ' ' && b[i] != '\t' && b[i] != '\r' && b[i] != '\n' {
			break
		}
	}
	j := len(b)
	for j > i {
		if b[j-1] != ' ' && b[j-1] != '\t' && b[j-1] != '\r' && b[j-1] != '\n' {
			break
		}
		j--
	}
	return b[i:j]
}
