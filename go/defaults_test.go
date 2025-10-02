package acp

import (
	"encoding/json"
	"testing"
)

// Ensure InitializeResponse.authMethods encodes to [] when nil or empty,
// and decodes to [] when missing or null.
func TestInitializeResponse_AuthMethods_Defaults(t *testing.T) {
	t.Parallel()
	t.Run("marshal_nil_slice_encodes_empty_array", func(t *testing.T) {
		t.Parallel()
		resp := InitializeResponse{ProtocolVersion: 1}
		b, err := json.Marshal(resp)
		if err != nil {
			t.Fatalf("marshal error: %v", err)
		}
		var m map[string]any
		if err := json.Unmarshal(b, &m); err != nil {
			t.Fatalf("roundtrip unmarshal error: %v", err)
		}
		v, ok := m["authMethods"]
		if !ok {
			t.Fatalf("authMethods missing in JSON: %s", string(b))
		}
		arr, ok := v.([]any)
		if !ok || len(arr) != 0 {
			t.Fatalf("authMethods should be empty array; got: %#v (json=%s)", v, string(b))
		}
	})

	t.Run("marshal_empty_slice_encodes_empty_array", func(t *testing.T) {
		t.Parallel()
		resp := InitializeResponse{ProtocolVersion: 1, AuthMethods: []AuthMethod{}}
		b, err := json.Marshal(resp)
		if err != nil {
			t.Fatalf("marshal error: %v", err)
		}
		var m map[string]any
		if err := json.Unmarshal(b, &m); err != nil {
			t.Fatalf("roundtrip unmarshal error: %v", err)
		}
		v, ok := m["authMethods"]
		if !ok {
			t.Fatalf("authMethods missing in JSON: %s", string(b))
		}
		arr, ok := v.([]any)
		if !ok || len(arr) != 0 {
			t.Fatalf("authMethods should be empty array; got: %#v (json=%s)", v, string(b))
		}
	})

	t.Run("unmarshal_missing_sets_empty_array", func(t *testing.T) {
		t.Parallel()
		var resp InitializeResponse
		if err := json.Unmarshal([]byte(`{"protocolVersion":1}`), &resp); err != nil {
			t.Fatalf("unmarshal error: %v", err)
		}
		if resp.AuthMethods == nil || len(resp.AuthMethods) != 0 {
			t.Fatalf("expected default empty authMethods; got: %#v", resp.AuthMethods)
		}
	})

	t.Run("unmarshal_null_sets_empty_array", func(t *testing.T) {
		t.Parallel()
		var resp InitializeResponse
		if err := json.Unmarshal([]byte(`{"protocolVersion":1, "authMethods": null}`), &resp); err != nil {
			t.Fatalf("unmarshal error: %v", err)
		}
		if resp.AuthMethods == nil || len(resp.AuthMethods) != 0 {
			t.Fatalf("expected default empty authMethods on null; got: %#v", resp.AuthMethods)
		}
	})
}

// Ensure InitializeRequest.clientCapabilities defaults apply on decode when missing,
// and that the property is present on encode even when zero-value.
func TestInitializeRequest_ClientCapabilities_Defaults(t *testing.T) {
	t.Parallel()
	t.Run("unmarshal_missing_applies_defaults", func(t *testing.T) {
		t.Parallel()
		var req InitializeRequest
		if err := json.Unmarshal([]byte(`{"protocolVersion":1}`), &req); err != nil {
			t.Fatalf("unmarshal error: %v", err)
		}
		// Defaults per schema: terminal=false; fs.readTextFile=false; fs.writeTextFile=false
		if req.ClientCapabilities.Terminal != false ||
			req.ClientCapabilities.Fs.ReadTextFile != false ||
			req.ClientCapabilities.Fs.WriteTextFile != false {
			t.Fatalf("unexpected clientCapabilities defaults: %+v", req.ClientCapabilities)
		}
	})

	t.Run("marshal_zero_includes_property", func(t *testing.T) {
		t.Parallel()
		req := InitializeRequest{ProtocolVersion: 1}
		b, err := json.Marshal(req)
		if err != nil {
			t.Fatalf("marshal error: %v", err)
		}
		var m map[string]any
		if err := json.Unmarshal(b, &m); err != nil {
			t.Fatalf("roundtrip unmarshal error: %v", err)
		}
		if _, ok := m["clientCapabilities"]; !ok {
			t.Fatalf("clientCapabilities should be present in JSON: %s", string(b))
		}
	})
}

// Ensure InitializeResponse.agentCapabilities defaults apply on decode when missing.
func TestInitializeResponse_AgentCapabilities_Defaults(t *testing.T) {
	t.Parallel()
	t.Run("unmarshal_missing_applies_defaults", func(t *testing.T) {
		t.Parallel()
		var resp InitializeResponse
		if err := json.Unmarshal([]byte(`{"protocolVersion":1}`), &resp); err != nil {
			t.Fatalf("unmarshal error: %v", err)
		}
		// Defaults: loadSession=false; promptCapabilities audio=false, embeddedContext=false, image=false
		if resp.AgentCapabilities.LoadSession != false ||
			resp.AgentCapabilities.PromptCapabilities.Audio != false ||
			resp.AgentCapabilities.PromptCapabilities.EmbeddedContext != false ||
			resp.AgentCapabilities.PromptCapabilities.Image != false {
			t.Fatalf("unexpected agentCapabilities defaults: %+v", resp.AgentCapabilities)
		}
	})
}
