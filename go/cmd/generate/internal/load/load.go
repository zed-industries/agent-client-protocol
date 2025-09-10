// Package load provides utilities to read the ACP JSON schema and
// accompanying metadata into minimal structures used by the generator.
package load

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

// Meta mirrors schema/meta.json for method maps and version.
type Meta struct {
	Version       int               `json:"version"`
	AgentMethods  map[string]string `json:"agentMethods"`
	ClientMethods map[string]string `json:"clientMethods"`
}

// Schema is a minimal view over schema/schema.json definitions used by the generator.
type Schema struct {
	Defs map[string]*Definition `json:"$defs"`
}

// Definition is a partial JSON Schema node the generator cares about.
type Definition struct {
	Description string                 `json:"description"`
	Type        any                    `json:"type"`
	Properties  map[string]*Definition `json:"properties"`
	Required    []string               `json:"required"`
	Enum        []any                  `json:"enum"`
	Items       *Definition            `json:"items"`
	Ref         string                 `json:"$ref"`
	AnyOf       []*Definition          `json:"anyOf"`
	OneOf       []*Definition          `json:"oneOf"`
	DocsIgnore  bool                   `json:"x-docs-ignore"`
	Title       string                 `json:"title"`
	Const       any                    `json:"const"`
	XSide       string                 `json:"x-side"`
	XMethod     string                 `json:"x-method"`
	// Default holds the JSON Schema default value, when present.
	// Used by generators to synthesize defaulting behavior.
	Default any `json:"default"`
}

// ReadMeta loads schema/meta.json.
func ReadMeta(schemaDir string) (*Meta, error) {
	metaBytes, err := os.ReadFile(filepath.Join(schemaDir, "meta.json"))
	if err != nil {
		return nil, fmt.Errorf("read meta.json: %w", err)
	}
	var meta Meta
	if err := json.Unmarshal(metaBytes, &meta); err != nil {
		return nil, fmt.Errorf("parse meta.json: %w", err)
	}
	return &meta, nil
}

// ReadSchema loads schema/schema.json.
func ReadSchema(schemaDir string) (*Schema, error) {
	schemaBytes, err := os.ReadFile(filepath.Join(schemaDir, "schema.json"))
	if err != nil {
		return nil, fmt.Errorf("read schema.json: %w", err)
	}
	var schema Schema
	if err := json.Unmarshal(schemaBytes, &schema); err != nil {
		return nil, fmt.Errorf("parse schema.json: %w", err)
	}
	return &schema, nil
}
