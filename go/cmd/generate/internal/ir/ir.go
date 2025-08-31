package ir

import (
	"sort"
	"strings"

	"github.com/zed-industries/agent-client-protocol/go/cmd/generate/internal/load"
	"github.com/zed-industries/agent-client-protocol/go/cmd/generate/internal/util"
)

// MethodBinding describes which interface a method belongs to on each side.
type MethodBinding int

const (
	BindUnknown MethodBinding = iota
	// Agent bindings
	BindAgent
	BindAgentLoader
	BindAgentExperimental
	// Client bindings
	BindClient
	BindClientExperimental
	BindClientTerminal
)

// MethodInfo captures association between a wire method and its Go types and binding.
type MethodInfo struct {
	Side        string // "agent" or "client"
	Method      string // wire method, e.g., "session/new"
	MethodKey   string // meta key, e.g., "session_new"
	Req         string // Go type name of Request
	Resp        string // Go type name of Response
	Notif       string // Go type name of Notification
	Binding     MethodBinding
	DocsIgnored bool
}

// Groups is a map keyed by side|wire to MethodInfo.
type Groups map[string]*MethodInfo

func key(side, method string) string { return side + "|" + method }

// PrimaryType mirrors logic from generator: find primary type string from a Definition.
func PrimaryType(d *load.Definition) string {
	if d == nil || d.Type == nil {
		return ""
	}
	switch v := d.Type.(type) {
	case string:
		return v
	case []any:
		var first string
		for _, e := range v {
			if s, ok := e.(string); ok {
				if first == "" {
					first = s
				}
				if s != "null" {
					return s
				}
			}
		}
		return first
	default:
		return ""
	}
}

// IsNullResponse returns true if the response schema is explicitly null or missing.
func IsNullResponse(def *load.Definition) bool {
	if def == nil {
		return true
	}
	if s, ok := def.Type.(string); ok && s == "null" {
		return true
	}
	return false
}

// BuildMethodGroups merges schema-provided links with meta fallback and returns groups.
func BuildMethodGroups(schema *load.Schema, meta *load.Meta) Groups {
	groups := Groups{}
	// From schema
	for name, def := range schema.Defs {
		if def == nil || def.XMethod == "" || def.XSide == "" {
			continue
		}
		k := key(def.XSide, def.XMethod)
		mi := groups[k]
		if mi == nil {
			mi = &MethodInfo{Side: def.XSide, Method: def.XMethod}
			groups[k] = mi
		}
		if strings.HasSuffix(name, "Request") {
			mi.Req = name
		}
		if strings.HasSuffix(name, "Response") {
			mi.Resp = name
		}
		if strings.HasSuffix(name, "Notification") {
			mi.Notif = name
		}
	}
	// From meta fallback (terminal etc.)
	for mk, wire := range meta.AgentMethods {
		k := key("agent", wire)
		if groups[k] == nil {
			base := inferTypeBaseFromMethodKey(mk)
			mi := &MethodInfo{Side: "agent", Method: wire}
			if wire == "session/cancel" {
				mi.Notif = "CancelNotification"
			} else {
				if _, ok := schema.Defs[base+"Request"]; ok {
					mi.Req = base + "Request"
				}
				if _, ok := schema.Defs[base+"Response"]; ok {
					mi.Resp = base + "Response"
				}
			}
			if mi.Req != "" || mi.Notif != "" {
				groups[k] = mi
			}
		}
	}
	for mk, wire := range meta.ClientMethods {
		k := key("client", wire)
		if groups[k] == nil {
			base := inferTypeBaseFromMethodKey(mk)
			mi := &MethodInfo{Side: "client", Method: wire}
			if wire == "session/update" {
				mi.Notif = "SessionNotification"
			} else {
				if _, ok := schema.Defs[base+"Request"]; ok {
					mi.Req = base + "Request"
				}
				if _, ok := schema.Defs[base+"Response"]; ok {
					mi.Resp = base + "Response"
				}
			}
			if mi.Req != "" || mi.Notif != "" {
				groups[k] = mi
			}
		}
	}
	// Post-process bindings and docs-ignore
	for _, mi := range groups {
		mi.Binding = classifyBinding(schema, meta, mi)
		mi.DocsIgnored = isDocsIgnoredMethod(schema, mi)
	}
	return groups
}

// classifyBinding determines interface binding for each method.
func classifyBinding(schema *load.Schema, meta *load.Meta, mi *MethodInfo) MethodBinding {
	if mi == nil {
		return BindUnknown
	}
	switch mi.Side {
	case "agent":
		if mi.Method == "session/load" {
			return BindAgentLoader
		}
		if isDocsIgnoredMethod(schema, mi) {
			return BindAgentExperimental
		}
		return BindAgent
	case "client":
		if isDocsIgnoredMethod(schema, mi) {
			if strings.HasPrefix(mi.Method, "terminal/") {
				return BindClientTerminal
			}
			return BindClientExperimental
		}
		return BindClient
	default:
		return BindUnknown
	}
}

// isDocsIgnoredMethod if any associated type (req/resp/notif) marked x-docs-ignore.
func isDocsIgnoredMethod(schema *load.Schema, mi *MethodInfo) bool {
	if mi == nil {
		return false
	}
	if mi.Req != "" {
		if d := schema.Defs[mi.Req]; d != nil && d.DocsIgnore {
			return true
		}
	}
	if mi.Resp != "" {
		if d := schema.Defs[mi.Resp]; d != nil && d.DocsIgnore {
			return true
		}
	}
	if mi.Notif != "" {
		if d := schema.Defs[mi.Notif]; d != nil && d.DocsIgnore {
			return true
		}
	}
	return false
}

// inferTypeBaseFromMethodKey mirrors previous heuristic; prefer schema when available.
func inferTypeBaseFromMethodKey(methodKey string) string {
	if methodKey == "terminal_wait_for_exit" {
		return "WaitForTerminalExit"
	}
	parts := strings.Split(methodKey, "_")
	if len(parts) == 2 {
		n, v := parts[0], parts[1]
		switch v {
		case "new", "create", "release", "wait", "load", "authenticate", "prompt", "cancel", "read", "write":
			return util.TitleWord(v) + util.TitleWord(n)
		default:
			return util.TitleWord(n) + util.TitleWord(v)
		}
	}
	segs := strings.Split(methodKey, "_")
	for i := range segs {
		segs[i] = util.TitleWord(segs[i])
	}
	return strings.Join(segs, "")
}

// DispatchMethodNameForNotification deduces trait method name for notifications.
func DispatchMethodNameForNotification(methodKey, typeName string) string {
	switch methodKey {
	case "session_update":
		return "SessionUpdate"
	case "session_cancel":
		return "Cancel"
	default:
		if strings.HasSuffix(typeName, "Notification") {
			return strings.TrimSuffix(typeName, "Notification")
		}
		return typeName
	}
}

// SortedKeys returns sorted keys of a map.
func SortedKeys(m map[string]string) []string {
	ks := make([]string, 0, len(m))
	for k := range m {
		ks = append(ks, k)
	}
	sort.Strings(ks)
	return ks
}
