package acp

import (
	"encoding/json"
	"fmt"
)

// RequestError represents a JSON-RPC error response.
type RequestError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
	Data    any    `json:"data,omitempty"`
}

func (e *RequestError) Error() string {
	// Prefer a structured, JSON-style string so callers get details by default
	// similar to the TypeScript client.
	// Example: {"code":-32603,"message":"Internal error","data":{"details":"..."}}
	if e == nil {
		return "<nil>"
	}
	// Try to pretty-print compact JSON for stability in logs.
	type view struct {
		Code    int    `json:"code"`
		Message string `json:"message"`
		Data    any    `json:"data,omitempty"`
	}
	v := view{Code: e.Code, Message: e.Message, Data: e.Data}
	b, err := json.Marshal(v)
	if err == nil {
		return string(b)
	}
	// Fallback if marshal fails.
	if e.Data != nil {
		return fmt.Sprintf("code %d: %s (data: %v)", e.Code, e.Message, e.Data)
	}
	return fmt.Sprintf("code %d: %s", e.Code, e.Message)
}

func NewParseError(data any) *RequestError {
	return &RequestError{Code: -32700, Message: "Parse error", Data: data}
}

func NewInvalidRequest(data any) *RequestError {
	return &RequestError{Code: -32600, Message: "Invalid request", Data: data}
}

func NewMethodNotFound(method string) *RequestError {
	return &RequestError{Code: -32601, Message: "Method not found", Data: map[string]any{"method": method}}
}

func NewInvalidParams(data any) *RequestError {
	return &RequestError{Code: -32602, Message: "Invalid params", Data: data}
}

func NewInternalError(data any) *RequestError {
	return &RequestError{Code: -32603, Message: "Internal error", Data: data}
}

func NewAuthRequired(data any) *RequestError {
	return &RequestError{Code: -32000, Message: "Authentication required", Data: data}
}

// toReqErr coerces arbitrary errors into JSON-RPC RequestError.
func toReqErr(err error) *RequestError {
	if err == nil {
		return nil
	}
	if re, ok := err.(*RequestError); ok {
		return re
	}
	return NewInternalError(map[string]any{"error": err.Error()})
}
