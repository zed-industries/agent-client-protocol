use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write;

pub struct MarkdownGenerator {
    definitions: BTreeMap<String, Value>,
    output: String,
}

impl MarkdownGenerator {
    pub fn new() -> Self {
        Self {
            definitions: BTreeMap::new(),
            output: String::new(),
        }
    }

    pub fn generate(&mut self, schema: &Value) -> String {
        // Extract definitions
        if let Some(defs) = schema.get("$defs").and_then(|v| v.as_object()) {
            self.definitions = defs.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        }

        // Start with title
        writeln!(&mut self.output, "---").unwrap();
        writeln!(&mut self.output, "title: \"Schema\"").unwrap();
        writeln!(
            &mut self.output,
            r#"description: "JSON Schema definitions for the Agent Client Protocol""#
        )
        .unwrap();
        writeln!(&mut self.output, "---").unwrap();
        writeln!(&mut self.output).unwrap();

        // Group definitions by their x-group metadata
        let mut grouped_types: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();

        for (name, def) in &self.definitions {
            let group = def
                .get("x-group")
                .and_then(|v| v.as_str())
                .unwrap_or("ungrouped")
                .to_string();

            grouped_types
                .entry(group)
                .or_default()
                .push((name.clone(), def.clone()));
        }

        // Sort groups by their order
        let mut groups: Vec<(String, Vec<(String, Value)>)> = grouped_types.into_iter().collect();
        groups.sort_by_key(|(group, _)| {
            match group.as_str() {
                "message-types" => 1,
                "initialization" => 2,
                "session" => 3,
                "content" => 4,
                "tools" => 5,
                "filesystem" => 6,
                "mcp" => 7,
                "ungrouped" => 999, // Show ungrouped types at the end
                _ => 1000,
            }
        });

        // Generate documentation for each group
        for (group, types) in groups {
            self.generate_group(&group, types);
        }

        self.output.clone()
    }

    fn generate_group(&mut self, group: &str, types: Vec<(String, Value)>) {
        // Generate section header
        let section_name = match group {
            "message-types" => "Message Types",
            "initialization" => "Initialization & Capabilities",
            "session" => "Session Management",
            "content" => "Content & Resources",
            "tools" => "Tools & Permissions",
            "filesystem" => "File System Operations",
            "mcp" => "MCP Server Configuration",
            "ungrouped" => "⚠️ Ungrouped Types (Missing Transform Attributes)",
            _ => group,
        };

        writeln!(&mut self.output, "## {}", section_name).unwrap();
        writeln!(&mut self.output).unwrap();

        // Add group description
        self.add_group_description(group);

        // Sort types within the group alphabetically
        let mut sorted_types = types;
        sorted_types.sort_by_key(|(name, _)| name.clone());

        // Generate documentation for each type
        for (name, def) in sorted_types {
            self.document_type(&name, &def);
        }
    }

    fn add_group_description(&mut self, group: &str) {
        let description = match group {
            "message-types" => {
                "Core protocol message types for communication between clients and agents."
            }
            "initialization" => {
                "Types for connection setup, capability negotiation, and authentication."
            }
            "session" => "Types for managing conversation sessions and prompt handling.",
            "content" => "Content blocks and resources that can be included in messages.",
            "tools" => "Tool execution, permissions, and planning capabilities.",
            "filesystem" => "File system access and manipulation operations.",
            "mcp" => "Model Context Protocol server configuration.",
            "ungrouped" => {
                "**Warning:** These types are missing `#[schemars(transform = ...)]` attributes and should be categorized."
            }
            _ => "",
        };

        if !description.is_empty() {
            writeln!(&mut self.output, "_{}_", description).unwrap();
            writeln!(&mut self.output).unwrap();
        }
    }

    fn document_type(&mut self, name: &str, definition: &Value) {
        writeln!(&mut self.output, "### {}", name).unwrap();
        writeln!(&mut self.output).unwrap();

        // Add main description if available
        if let Some(desc) = definition.get("description").and_then(|v| v.as_str()) {
            writeln!(&mut self.output, "{}", desc).unwrap();
            writeln!(&mut self.output).unwrap();
        }

        // Determine type kind and document accordingly
        if definition.get("oneOf").is_some() || definition.get("anyOf").is_some() {
            self.document_union(definition);
        } else if definition.get("enum").is_some() {
            self.document_enum(definition);
        } else if definition.get("properties").is_some() {
            self.document_object(definition);
        } else if let Some(type_val) = definition.get("type").and_then(|v| v.as_str()) {
            self.document_simple_type(type_val, definition);
        }

        writeln!(&mut self.output, "---").unwrap();
        writeln!(&mut self.output).unwrap();
    }

    fn document_union(&mut self, definition: &Value) {
        writeln!(&mut self.output, "**Type:** Union").unwrap();
        writeln!(&mut self.output).unwrap();

        let variants = definition
            .get("oneOf")
            .or_else(|| definition.get("anyOf"))
            .and_then(|v| v.as_array());

        if let Some(variants) = variants {
            writeln!(&mut self.output, "**Variants:**").unwrap();
            writeln!(&mut self.output).unwrap();

            for variant in variants {
                write!(&mut self.output, "- ").unwrap();
                self.document_variant(variant);
                writeln!(&mut self.output).unwrap();
            }
        }
    }

    fn document_variant(&mut self, variant: &Value) {
        // Check for $ref
        if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
            write!(&mut self.output, "`{}`", type_name).unwrap();

            // Add description from title if available
            if let Some(title) = variant.get("title").and_then(|v| v.as_str()) {
                write!(&mut self.output, " - {}", title).unwrap();
            }
            return;
        }

        // Check for const value
        if let Some(const_val) = variant.get("const") {
            if let Some(s) = const_val.as_str() {
                write!(&mut self.output, "`\"{}\"`", s).unwrap();
            } else {
                write!(&mut self.output, "`{}`", const_val).unwrap();
            }

            if let Some(desc) = variant.get("description").and_then(|v| v.as_str()) {
                write!(&mut self.output, " - {}", desc).unwrap();
            }
            return;
        }

        // Check for null type (e.g., AuthenticateResponse, LoadSessionResponse)
        if variant.get("type").and_then(|v| v.as_str()) == Some("null") {
            if let Some(title) = variant.get("title").and_then(|v| v.as_str()) {
                write!(&mut self.output, "`{}` (empty response)", title).unwrap();
            } else {
                write!(&mut self.output, "`null`").unwrap();
            }
            return;
        }

        // Check for inline object with discriminator
        if let Some(props) = variant.get("properties").and_then(|v| v.as_object()) {
            // Look for any property with a const value to use as discriminator
            let discriminator = props
                .iter()
                .find(|(_, v)| v.get("const").is_some())
                .and_then(|(_, v)| v.get("const").and_then(|c| c.as_str()));

            if let Some(const_val) = discriminator {
                write!(&mut self.output, "`{}` variant", const_val).unwrap();

                if let Some(desc) = variant.get("description").and_then(|v| v.as_str()) {
                    write!(&mut self.output, " - {}", desc).unwrap();
                }
            }
        }
    }

    fn document_enum(&mut self, definition: &Value) {
        if let Some(enum_vals) = definition.get("enum").and_then(|v| v.as_array()) {
            writeln!(&mut self.output, "**Type:** Enumeration").unwrap();
            writeln!(&mut self.output).unwrap();
            writeln!(&mut self.output, "**Allowed Values:**").unwrap();
            writeln!(&mut self.output).unwrap();

            for val in enum_vals {
                if let Some(s) = val.as_str() {
                    writeln!(&mut self.output, "- `\"{}\"`", s).unwrap();
                } else {
                    writeln!(&mut self.output, "- `{}`", val).unwrap();
                }
            }
        }
    }

    fn document_object(&mut self, definition: &Value) {
        writeln!(&mut self.output, "**Type:** Object").unwrap();

        if let Some(props) = definition.get("properties").and_then(|v| v.as_object()) {
            if props.is_empty() {
                return;
            }

            writeln!(&mut self.output).unwrap();
            writeln!(&mut self.output, "**Properties:**").unwrap();
            writeln!(&mut self.output).unwrap();

            // Get required fields
            let required = definition
                .get("required")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();

            // Create a table for properties
            writeln!(
                &mut self.output,
                "| Property | Type | Required | Description |"
            )
            .unwrap();
            writeln!(
                &mut self.output,
                "|----------|------|----------|-------------|"
            )
            .unwrap();

            // Sort properties for consistent output
            let mut sorted_props: Vec<(&String, &Value)> = props.iter().collect();
            sorted_props.sort_by_key(|(name, _)| name.as_str());

            for (prop_name, prop_schema) in sorted_props {
                let is_required = required.contains(&prop_name.as_str());
                let type_str = self.get_type_string(prop_schema);
                let desc = prop_schema
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .replace('\n', " ")
                    .replace('|', "&#124;"); // Escape pipes for markdown tables using HTML entity

                writeln!(
                    &mut self.output,
                    "| `{}` | `{}` | {} | {} |",
                    prop_name,
                    type_str.replace('|', "&#124;"), // Escape pipes in type string using HTML entity
                    if is_required { "✓" } else { "" },
                    desc
                )
                .unwrap();
            }

            // Add detailed property documentation if there are complex properties
            let complex_props: Vec<(&String, &Value)> = props
                .iter()
                .filter(|(_, schema)| {
                    // Check if property has additional constraints or is complex
                    schema.get("properties").is_some()
                        || schema.get("oneOf").is_some()
                        || schema.get("anyOf").is_some()
                        || schema.get("enum").is_some()
                        || schema.get("default").is_some()
                        || schema.get("minimum").is_some()
                        || schema.get("maximum").is_some()
                        || schema.get("minLength").is_some()
                        || schema.get("maxLength").is_some()
                        || schema.get("pattern").is_some()
                })
                .collect();

            if !complex_props.is_empty() {
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "**Property Details:**").unwrap();
                writeln!(&mut self.output).unwrap();

                for (prop_name, prop_schema) in complex_props {
                    writeln!(&mut self.output, "#### `{}`", prop_name).unwrap();

                    // Add constraints
                    self.document_constraints(prop_schema);
                    writeln!(&mut self.output).unwrap();
                }
            }
        }
    }

    fn document_simple_type(&mut self, type_name: &str, definition: &Value) {
        let formatted_type = match type_name {
            "integer" => {
                if let Some(format) = definition.get("format").and_then(|v| v.as_str()) {
                    format!("integer ({})", format)
                } else {
                    "integer".to_string()
                }
            }
            "number" => {
                if let Some(format) = definition.get("format").and_then(|v| v.as_str()) {
                    format!("number ({})", format)
                } else {
                    "number".to_string()
                }
            }
            "string" => {
                if let Some(format) = definition.get("format").and_then(|v| v.as_str()) {
                    format!("string ({})", format)
                } else {
                    "string".to_string()
                }
            }
            _ => type_name.to_string(),
        };

        writeln!(&mut self.output, "**Type:** `{}`", formatted_type).unwrap();

        // Document constraints if any
        self.document_constraints(definition);
    }

    fn document_constraints(&mut self, schema: &Value) {
        let mut constraints = Vec::new();

        if let Some(v) = schema.get("default") {
            constraints.push(format!(
                "Default: `{}`",
                serde_json::to_string(v).unwrap_or_default()
            ));
        }
        if let Some(v) = schema.get("minimum") {
            constraints.push(format!("Minimum: `{}`", v));
        }
        if let Some(v) = schema.get("maximum") {
            constraints.push(format!("Maximum: `{}`", v));
        }
        if let Some(v) = schema.get("minLength") {
            constraints.push(format!("Min length: `{}`", v));
        }
        if let Some(v) = schema.get("maxLength") {
            constraints.push(format!("Max length: `{}`", v));
        }
        if let Some(v) = schema.get("pattern") {
            constraints.push(format!("Pattern: `{}`", v));
        }
        if let Some(v) = schema.get("format").and_then(|v| v.as_str())
            && !["int32", "int64", "uint16", "uint32", "uint64", "double"].contains(&v)
        {
            constraints.push(format!("Format: `{}`", v));
        }

        if !constraints.is_empty() {
            writeln!(&mut self.output).unwrap();
            writeln!(&mut self.output, "**Constraints:**").unwrap();
            for constraint in constraints {
                writeln!(&mut self.output, "- {}", constraint).unwrap();
            }
        }

        // Document enum values if present
        if let Some(enum_vals) = schema.get("enum").and_then(|v| v.as_array()) {
            writeln!(&mut self.output).unwrap();
            writeln!(&mut self.output, "**Allowed values:**").unwrap();
            for val in enum_vals {
                if let Some(s) = val.as_str() {
                    writeln!(&mut self.output, "- `\"{}\"`", s).unwrap();
                } else {
                    writeln!(&mut self.output, "- `{}`", val).unwrap();
                }
            }
        }
    }

    fn get_type_string(&self, schema: &Value) -> String {
        // Check for $ref
        if let Some(ref_val) = schema.get("$ref").and_then(|v| v.as_str()) {
            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
            return type_name.to_string();
        }

        // Check for type
        if let Some(type_val) = schema.get("type") {
            if let Some(type_str) = type_val.as_str() {
                return match type_str {
                    "array" => {
                        if let Some(items) = schema.get("items") {
                            let item_type = self.get_type_string(items);
                            format!("{}[]", item_type)
                        } else {
                            "array".to_string()
                        }
                    }
                    "integer" => {
                        if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
                            format
                        } else {
                            type_str
                        }
                        .to_string()
                    }
                    _ => type_str.to_string(),
                };
            }

            // Handle multiple types (nullable)
            if let Some(arr) = type_val.as_array() {
                let types: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if !types.is_empty() {
                    return types.join(" | "); // Keep pipes as-is, will be escaped when used in tables
                }
            }
        }

        // Check for oneOf/anyOf
        if schema.get("oneOf").is_some() || schema.get("anyOf").is_some() {
            // Try to get more specific union type info
            if let Some(variants) = schema.get("oneOf").or_else(|| schema.get("anyOf"))
                && let Some(arr) = variants.as_array()
                && arr.len() == 2
            {
                // Check for nullable pattern (type | null)
                let mut has_null = false;
                let mut other_type = None;
                for variant in arr {
                    if variant.get("type").and_then(|v| v.as_str()) == Some("null") {
                        has_null = true;
                    } else if let Some(t) = self.get_inline_variant_type(variant) {
                        other_type = Some(t);
                    }
                }
                if has_null && other_type.is_some() {
                    return format!("{} | null", other_type.unwrap());
                }
            }
            return "union".to_string();
        }

        // Check for enum
        if schema.get("enum").is_some() {
            return "enum".to_string();
        }

        "object".to_string()
    }

    fn get_inline_variant_type(&self, variant: &Value) -> Option<String> {
        // Check for simple type
        if let Some(type_str) = variant.get("type").and_then(|v| v.as_str()) {
            return Some(type_str.to_string());
        }
        // Check for $ref
        if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
            return Some(type_name.to_string());
        }
        None
    }
}
