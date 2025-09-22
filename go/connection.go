package acp

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"log/slog"
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

	ctx    context.Context
	cancel context.CancelCauseFunc

	logger *slog.Logger
}

func NewConnection(handler MethodHandler, peerInput io.Writer, peerOutput io.Reader) *Connection {
	ctx, cancel := context.WithCancelCause(context.Background())
	c := &Connection{
		w:       peerInput,
		r:       peerOutput,
		handler: handler,
		pending: make(map[string]*pendingResponse),
		ctx:     ctx,
		cancel:  cancel,
	}
	go c.receive()
	return c
}

// SetLogger installs a logger used for internal connection diagnostics.
// If unset, logs are written via the default logger.
func (c *Connection) SetLogger(l *slog.Logger) { c.logger = l }

func (c *Connection) loggerOrDefault() *slog.Logger {
	if c.logger != nil {
		return c.logger
	}
	return slog.Default()
}

func (c *Connection) receive() {
	const (
		initialBufSize = 1024 * 1024
		maxBufSize     = 10 * 1024 * 1024
	)

	scanner := bufio.NewScanner(c.r)
	buf := make([]byte, 0, initialBufSize)
	scanner.Buffer(buf, maxBufSize)

	for scanner.Scan() {
		line := scanner.Bytes()
		if len(bytes.TrimSpace(line)) == 0 {
			continue
		}

		var msg anyMessage
		if err := json.Unmarshal(line, &msg); err != nil {
			c.loggerOrDefault().Error("failed to parse incoming message", "err", err, "raw", string(line))
			continue
		}

		switch {
		case msg.ID != nil && msg.Method == "":
			c.handleResponse(&msg)
		case msg.Method != "":
			go c.handleInbound(&msg)
		default:
			c.loggerOrDefault().Error("received message with neither id nor method", "raw", string(line))
		}
	}

	c.cancel(errors.New("peer connection closed"))
	c.loggerOrDefault().Info("peer connection closed")
}

func (c *Connection) handleResponse(msg *anyMessage) {
	idStr := string(*msg.ID)

	c.mu.Lock()
	pr := c.pending[idStr]
	if pr != nil {
		delete(c.pending, idStr)
	}
	c.mu.Unlock()

	if pr != nil {
		pr.ch <- *msg
	}
}

func (c *Connection) handleInbound(req *anyMessage) {
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

	result, err := c.handler(c.ctx, req.Method, req.Params)
	if req.ID == nil {
		// Notification: no response is sent; log handler errors to surface decode failures.
		if err != nil {
			c.loggerOrDefault().Error("failed to handle notification", "method", req.Method, "err", err)
		}
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
	var result T

	msg, idKey, err := c.prepareRequest(method, params)
	if err != nil {
		return result, err
	}

	pr := &pendingResponse{ch: make(chan anyMessage, 1)}
	c.mu.Lock()
	c.pending[idKey] = pr
	c.mu.Unlock()

	if err := c.sendMessage(msg); err != nil {
		c.cleanupPending(idKey)
		return result, NewInternalError(map[string]any{"error": err.Error()})
	}

	resp, err := c.waitForResponse(ctx, pr, idKey)
	if err != nil {
		return result, err
	}

	if resp.Error != nil {
		return result, resp.Error
	}

	if len(resp.Result) > 0 {
		if err := json.Unmarshal(resp.Result, &result); err != nil {
			return result, NewInternalError(map[string]any{"error": err.Error()})
		}
	}
	return result, nil
}

func (c *Connection) prepareRequest(method string, params any) (anyMessage, string, error) {
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
			return msg, "", NewInvalidParams(map[string]any{"error": err.Error()})
		}
		msg.Params = b
	}

	return msg, string(idRaw), nil
}

func (c *Connection) waitForResponse(ctx context.Context, pr *pendingResponse, idKey string) (anyMessage, error) {
	select {
	case resp := <-pr.ch:
		return resp, nil
	case <-ctx.Done():
		c.cleanupPending(idKey)
		return anyMessage{}, NewInternalError(map[string]any{"error": context.Cause(ctx).Error()})
	case <-c.Done():
		return anyMessage{}, NewInternalError(map[string]any{"error": "peer disconnected before response"})
	}
}

func (c *Connection) cleanupPending(idKey string) {
	c.mu.Lock()
	delete(c.pending, idKey)
	c.mu.Unlock()
}

// SendRequestNoResult sends a JSON-RPC request that returns no result payload.
func (c *Connection) SendRequestNoResult(ctx context.Context, method string, params any) error {
	msg, idKey, err := c.prepareRequest(method, params)
	if err != nil {
		return err
	}

	pr := &pendingResponse{ch: make(chan anyMessage, 1)}
	c.mu.Lock()
	c.pending[idKey] = pr
	c.mu.Unlock()

	if err := c.sendMessage(msg); err != nil {
		c.cleanupPending(idKey)
		return NewInternalError(map[string]any{"error": err.Error()})
	}

	resp, err := c.waitForResponse(ctx, pr, idKey)
	if err != nil {
		return err
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

	msg, err := c.prepareNotification(method, params)
	if err != nil {
		return err
	}

	if err := c.sendMessage(msg); err != nil {
		return NewInternalError(map[string]any{"error": err.Error()})
	}
	return nil
}

func (c *Connection) prepareNotification(method string, params any) (anyMessage, error) {
	msg := anyMessage{
		JSONRPC: "2.0",
		Method:  method,
	}

	if params != nil {
		b, err := json.Marshal(params)
		if err != nil {
			return msg, NewInvalidParams(map[string]any{"error": err.Error()})
		}
		msg.Params = b
	}

	return msg, nil
}

// Done returns a channel that is closed when the underlying reader loop exits
// (typically when the peer disconnects or the input stream is closed).
func (c *Connection) Done() <-chan struct{} {
	return c.ctx.Done()
}
