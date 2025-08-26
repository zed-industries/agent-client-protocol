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

        // Start with title and frontmatter
        writeln!(&mut self.output, "---").unwrap();
        writeln!(&mut self.output, "title: \"Schema\"").unwrap();
        writeln!(
            &mut self.output,
            r#"description: "JSON Schema definitions for the Agent Client Protocol""#
        )
        .unwrap();
        writeln!(&mut self.output, "---").unwrap();
        writeln!(&mut self.output).unwrap();

        // Add introduction with callout
        writeln!(&mut self.output, "<Info>").unwrap();
        writeln!(&mut self.output, "  This documentation provides comprehensive schema definitions for all types used in the Agent Client Protocol.").unwrap();
        writeln!(
            &mut self.output,
            "  Each type includes detailed property descriptions, constraints, and usage examples."
        )
        .unwrap();
        writeln!(&mut self.output, "</Info>").unwrap();
        writeln!(&mut self.output).unwrap();

        // Add an overview section with cards
        self.generate_overview();

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
        groups.sort_by_key(|(group, _)| match group.as_str() {
            "message-types" => 1,
            "initialization" => 2,
            "session" => 3,
            "content" => 4,
            "tools" => 5,
            "filesystem" => 6,
            "mcp" => 7,
            "ungrouped" => 999, // Show ungrouped types at the end
            _ => 1000,
        });

        // Generate documentation for each group
        for (group, types) in groups {
            self.generate_group(&group, types);
        }

        self.output.clone()
    }

    fn generate_overview(&mut self) {
        writeln!(&mut self.output, "## Overview").unwrap();
        writeln!(&mut self.output).unwrap();

        writeln!(&mut self.output, "<CardGroup cols={{2}}>").unwrap();

        writeln!(
            &mut self.output,
            "  <Card title=\"Message Types\" icon=\"message\" href=\"#message-types\">"
        )
        .unwrap();
        writeln!(
            &mut self.output,
            "    Core protocol message types for client-agent communication"
        )
        .unwrap();
        writeln!(&mut self.output, "  </Card>").unwrap();

        writeln!(&mut self.output, "  <Card title=\"Initialization\" icon=\"rocket\" href=\"#initialization--capabilities\">").unwrap();
        writeln!(
            &mut self.output,
            "    Connection setup, capability negotiation, and authentication"
        )
        .unwrap();
        writeln!(&mut self.output, "  </Card>").unwrap();

        writeln!(
            &mut self.output,
            "  <Card title=\"Session Management\" icon=\"users\" href=\"#session-management\">"
        )
        .unwrap();
        writeln!(
            &mut self.output,
            "    Managing conversation sessions and prompts"
        )
        .unwrap();
        writeln!(&mut self.output, "  </Card>").unwrap();

        writeln!(
            &mut self.output,
            "  <Card title=\"Content & Resources\" icon=\"file-text\" href=\"#content--resources\">"
        )
        .unwrap();
        writeln!(
            &mut self.output,
            "    Content blocks and resources in messages"
        )
        .unwrap();
        writeln!(&mut self.output, "  </Card>").unwrap();

        writeln!(
            &mut self.output,
            "  <Card title=\"Tools & Permissions\" icon=\"tool\" href=\"#tools--permissions\">"
        )
        .unwrap();
        writeln!(
            &mut self.output,
            "    Tool execution and permission management"
        )
        .unwrap();
        writeln!(&mut self.output, "  </Card>").unwrap();

        writeln!(
            &mut self.output,
            "  <Card title=\"File System\" icon=\"folder\" href=\"#file-system-operations\">"
        )
        .unwrap();
        writeln!(&mut self.output, "    File system access and manipulation").unwrap();
        writeln!(&mut self.output, "  </Card>").unwrap();

        writeln!(&mut self.output, "</CardGroup>").unwrap();
        writeln!(&mut self.output).unwrap();
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
            "ungrouped" => "Other Types",
            _ => group,
        };

        writeln!(&mut self.output, "## {}", section_name).unwrap();
        writeln!(&mut self.output).unwrap();

        // Add group description in a callout
        // Add group description
        self.add_group_description(group);

        // Sort types within the group alphabetically
        let mut sorted_types = types;
        sorted_types.sort_by_key(|(name, _)| name.clone());

        // Document each type in this section
        for (name, def) in sorted_types {
            self.document_type(&name, &def);
        }

        // Add horizontal rule after each major section
        writeln!(&mut self.output, "---").unwrap();
        writeln!(&mut self.output).unwrap();
    }

    fn add_group_description(&mut self, group: &str) {
        let (callout_type, description) = match group {
            "message-types" => (
                "Info",
                "Core protocol message types for communication between clients and agents.",
            ),
            "initialization" => (
                "Info",
                "Types for connection setup, capability negotiation, and authentication.",
            ),
            "session" => (
                "Info",
                "Types for managing conversation sessions and prompt handling.",
            ),
            "content" => (
                "Info",
                "Content blocks and resources that can be included in messages.",
            ),
            "tools" => (
                "Info",
                "Tool execution, permissions, and planning capabilities.",
            ),
            "filesystem" => ("Info", "File system access and manipulation operations."),
            "mcp" => ("Info", "Model Context Protocol server configuration."),
            "ungrouped" => (
                "Warning",
                "These types are missing categorization metadata.",
            ),
            _ => return,
        };

        writeln!(&mut self.output, "<{}>", callout_type).unwrap();
        writeln!(&mut self.output, "  {}", description).unwrap();
        writeln!(&mut self.output, "</{}>", callout_type).unwrap();
        writeln!(&mut self.output).unwrap();
    }

    fn document_type(&mut self, name: &str, definition: &Value) {
        writeln!(&mut self.output, "### {}", name).unwrap();
        writeln!(&mut self.output).unwrap();

        // Add main description if available
        if let Some(desc) = definition.get("description").and_then(|v| v.as_str()) {
            // Escape # at the beginning of lines to prevent them from being treated as headers
            let escaped_desc = self.escape_description(desc);
            writeln!(&mut self.output, "{}", escaped_desc).unwrap();
            writeln!(&mut self.output).unwrap();
        }
        // Determine type kind and document accordingly
        if definition.get("oneOf").is_some() || definition.get("anyOf").is_some() {
            self.document_union(definition);
        } else if definition.get("enum").is_some() {
            self.document_enum_simple(definition);
        } else if definition.get("properties").is_some() {
            self.document_object(definition);
        } else if let Some(type_val) = definition.get("type").and_then(|v| v.as_str()) {
            self.document_simple_type(type_val, definition);
        }

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
            writeln!(&mut self.output, "| Variant | Description |").unwrap();
            writeln!(&mut self.output, "| ------- | ----------- |").unwrap();

            for variant in variants {
                self.document_variant_table_row(variant);
            }
            writeln!(&mut self.output).unwrap();
        }
    }

    fn document_variant_table_row(&mut self, variant: &Value) {
        write!(&mut self.output, "| ").unwrap();

        // Get variant name
        if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
            write!(&mut self.output, "`{}`", type_name).unwrap();
        } else if let Some(const_val) = variant.get("const") {
            if let Some(s) = const_val.as_str() {
                write!(&mut self.output, "`\"{}\"`", s).unwrap();
            } else {
                write!(&mut self.output, "`{}`", const_val).unwrap();
            }
        } else if variant.get("type").and_then(|v| v.as_str()) == Some("null") {
            write!(&mut self.output, "`null`").unwrap();
        } else if let Some(props) = variant.get("properties").and_then(|v| v.as_object()) {
            // Look for discriminator
            let discriminator = props
                .iter()
                .find(|(_, v)| v.get("const").is_some())
                .and_then(|(_, v)| v.get("const").and_then(|c| c.as_str()));

            if let Some(const_val) = discriminator {
                write!(&mut self.output, "`{}` variant", const_val).unwrap();
            } else {
                write!(&mut self.output, "Object").unwrap();
            }
        } else {
            write!(&mut self.output, "Variant").unwrap();
        }

        write!(&mut self.output, " | ").unwrap();

        // Get description
        if let Some(title) = variant.get("title").and_then(|v| v.as_str()) {
            let escaped_title = self.escape_mdx(title);
            write!(&mut self.output, "{}", escaped_title).unwrap();
        } else if let Some(desc) = variant.get("description").and_then(|v| v.as_str()) {
            let escaped_desc = self.escape_mdx(desc);
            write!(&mut self.output, "{}", escaped_desc).unwrap();
        } else if variant.get("type").and_then(|v| v.as_str()) == Some("null") {
            write!(&mut self.output, "Empty response").unwrap();
        } else if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
            write!(&mut self.output, "{}", type_name).unwrap();
        } else {
            write!(&mut self.output, "-").unwrap();
        }

        writeln!(&mut self.output, " |").unwrap();
    }

    fn document_enum_simple(&mut self, definition: &Value) {
        if let Some(enum_vals) = definition.get("enum").and_then(|v| v.as_array()) {
            writeln!(&mut self.output, "**Type:** Enumeration").unwrap();
            writeln!(&mut self.output).unwrap();

            writeln!(&mut self.output, "| Value |").unwrap();
            writeln!(&mut self.output, "| ----- |").unwrap();

            for val in enum_vals {
                write!(&mut self.output, "| ").unwrap();
                if let Some(s) = val.as_str() {
                    write!(&mut self.output, "`\"{}\"`", s).unwrap();
                } else {
                    write!(&mut self.output, "`{}`", val).unwrap();
                }
                writeln!(&mut self.output, " |").unwrap();
            }
            writeln!(&mut self.output).unwrap();
        }
    }

    fn document_object(&mut self, definition: &Value) {
        writeln!(&mut self.output, "**Type:** Object").unwrap();

        if let Some(props) = definition.get("properties").and_then(|v| v.as_object()) {
            if props.is_empty() {
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "*No properties defined*").unwrap();
                return;
            }

            writeln!(&mut self.output).unwrap();
            writeln!(&mut self.output, "**Properties:**").unwrap();
            writeln!(&mut self.output).unwrap();
            self.document_properties_as_fields(props, definition, 0);
        }
    }

    fn document_properties_as_fields(
        &mut self,
        props: &serde_json::Map<String, Value>,
        definition: &Value,
        indent: usize,
    ) {
        let indent_str = " ".repeat(indent);

        // Get required fields
        let required = definition
            .get("required")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        // Sort properties for consistent output
        let mut sorted_props: Vec<(&String, &Value)> = props.iter().collect();
        sorted_props.sort_by_key(|(name, _)| name.as_str());

        for (prop_name, prop_schema) in sorted_props {
            let is_required = required.contains(&prop_name.as_str());
            let type_str = self.get_type_string(prop_schema);

            // Check if this property has nested properties
            let has_nested =
                prop_schema.get("properties").is_some() || self.is_complex_ref(prop_schema);

            if has_nested {
                writeln!(
                    &mut self.output,
                    "{}<ResponseField name=\"{}\" type=\"{}\" {}>",
                    indent_str,
                    prop_name,
                    type_str,
                    if is_required { "required" } else { "" }
                )
                .unwrap();

                // Add description if available
                if let Some(desc) = prop_schema.get("description").and_then(|v| v.as_str()) {
                    let escaped_desc = self.escape_mdx(desc);
                    writeln!(&mut self.output, "{}  {}", indent_str, escaped_desc).unwrap();
                    writeln!(&mut self.output).unwrap();
                }

                // Add expandable for nested properties
                writeln!(
                    &mut self.output,
                    "{}  <Expandable title=\"properties\">",
                    indent_str
                )
                .unwrap();

                if let Some(nested_props) =
                    prop_schema.get("properties").and_then(|v| v.as_object())
                {
                    self.document_properties_as_fields(nested_props, prop_schema, indent + 4);
                } else if let Some(ref_val) = prop_schema.get("$ref").and_then(|v| v.as_str()) {
                    let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                    if let Some(ref_def) = self.definitions.get(type_name).cloned()
                        && let Some(nested_props) =
                            ref_def.get("properties").and_then(|v| v.as_object())
                    {
                        self.document_properties_as_fields(nested_props, &ref_def, indent + 4);
                    }
                }

                writeln!(&mut self.output, "{}  </Expandable>", indent_str).unwrap();
                writeln!(&mut self.output, "{}</ResponseField>", indent_str).unwrap();
            } else {
                // Simple field without nesting
                writeln!(
                    &mut self.output,
                    "{}<ResponseField name=\"{}\" type=\"{}\" {}>",
                    indent_str,
                    prop_name,
                    type_str,
                    if is_required { "required" } else { "" }
                )
                .unwrap();

                // Add description if available
                if let Some(desc) = prop_schema.get("description").and_then(|v| v.as_str()) {
                    let escaped_desc = self.escape_mdx(desc);
                    writeln!(&mut self.output, "{}  {}", indent_str, escaped_desc).unwrap();
                }

                // Add constraints if any
                self.document_field_constraints(prop_schema, indent + 2);

                writeln!(&mut self.output, "{}</ResponseField>", indent_str).unwrap();
            }
        }
    }

    fn document_field_constraints(&mut self, schema: &Value, indent: usize) {
        let indent_str = " ".repeat(indent);
        let mut constraints = Vec::new();

        if let Some(v) = schema.get("default") {
            constraints.push((
                "Default",
                format!("`{}`", serde_json::to_string(v).unwrap_or_default()),
            ));
        }
        if let Some(v) = schema.get("minimum") {
            constraints.push(("Minimum", format!("`{}`", v)));
        }
        if let Some(v) = schema.get("maximum") {
            constraints.push(("Maximum", format!("`{}`", v)));
        }
        if let Some(v) = schema.get("minLength") {
            constraints.push(("Min length", format!("`{}`", v)));
        }
        if let Some(v) = schema.get("maxLength") {
            constraints.push(("Max length", format!("`{}`", v)));
        }
        if let Some(v) = schema.get("pattern") {
            constraints.push(("Pattern", format!("`{}`", v)));
        }

        if !constraints.is_empty() {
            writeln!(&mut self.output).unwrap();
            if constraints.len() == 1 {
                // Single constraint as text
                let (name, value) = &constraints[0];
                writeln!(&mut self.output, "{}  - {}: {}", indent_str, name, value).unwrap();
            } else {
                // Multiple constraints as table
                writeln!(&mut self.output, "{}  | Constraint | Value |", indent_str).unwrap();
                writeln!(&mut self.output, "{}  | ---------- | ----- |", indent_str).unwrap();
                for (name, value) in constraints {
                    writeln!(&mut self.output, "{}  | {} | {} |", indent_str, name, value).unwrap();
                }
            }
        }

        // Document enum values if present
        if let Some(enum_vals) = schema.get("enum").and_then(|v| v.as_array()) {
            writeln!(&mut self.output).unwrap();
            writeln!(&mut self.output, "{}  **Allowed values:**", indent_str).unwrap();
            for val in enum_vals {
                if let Some(s) = val.as_str() {
                    writeln!(&mut self.output, "{}  - `\"{}\"`", indent_str, s).unwrap();
                } else {
                    writeln!(&mut self.output, "{}  - `{}`", indent_str, val).unwrap();
                }
            }
        }
    }

    fn is_complex_ref(&self, schema: &Value) -> bool {
        if let Some(ref_val) = schema.get("$ref").and_then(|v| v.as_str()) {
            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
            if let Some(def) = self.definitions.get(type_name) {
                return def.get("properties").is_some();
            }
        }
        false
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
            constraints.push((
                "Default",
                format!("`{}`", serde_json::to_string(v).unwrap_or_default()),
            ));
        }
        if let Some(v) = schema.get("minimum") {
            constraints.push(("Minimum", format!("`{}`", v)));
        }
        if let Some(v) = schema.get("maximum") {
            constraints.push(("Maximum", format!("`{}`", v)));
        }
        if let Some(v) = schema.get("minLength") {
            constraints.push(("Min length", format!("`{}`", v)));
        }
        if let Some(v) = schema.get("maxLength") {
            constraints.push(("Max length", format!("`{}`", v)));
        }
        if let Some(v) = schema.get("pattern") {
            constraints.push(("Pattern", format!("`{}`", v)));
        }
        if let Some(v) = schema.get("format").and_then(|v| v.as_str())
            && !["int32", "int64", "uint16", "uint32", "uint64", "double"].contains(&v)
        {
            constraints.push(("Format", format!("`{}`", v)));
        }

        if !constraints.is_empty() {
            writeln!(&mut self.output).unwrap();
            if constraints.len() == 1 {
                // Single constraint as text
                let (name, value) = &constraints[0];
                writeln!(&mut self.output, "**{}:** {}", name, value).unwrap();
            } else {
                // Multiple constraints as table
                writeln!(&mut self.output, "| Constraint | Value |").unwrap();
                writeln!(&mut self.output, "| ---------- | ----- |").unwrap();
                for (name, value) in constraints {
                    writeln!(&mut self.output, "| {} | {} |", name, value).unwrap();
                }
            }
        }

        // Document enum values if present
        if let Some(enum_vals) = schema.get("enum").and_then(|v| v.as_array()) {
            writeln!(&mut self.output).unwrap();
            writeln!(&mut self.output, "| Allowed Values |").unwrap();
            writeln!(&mut self.output, "| -------------- |").unwrap();
            for val in enum_vals {
                write!(&mut self.output, "| ").unwrap();
                if let Some(s) = val.as_str() {
                    write!(&mut self.output, "`\"{}\"`", s).unwrap();
                } else {
                    write!(&mut self.output, "`{}`", val).unwrap();
                }
                writeln!(&mut self.output, " |").unwrap();
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
                    return types.join(" | ");
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

    fn escape_mdx(&self, text: &str) -> String {
        text.replace('|', "\\|")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('{', "\\{")
            .replace('}', "\\}")
    }

    fn escape_description(&self, text: &str) -> String {
        // Escape # at the beginning of lines to prevent them from being treated as headers
        let lines: Vec<String> = text
            .lines()
            .map(|line| {
                if line.trim_start().starts_with('#') {
                    // Escape the # by replacing it with \#
                    let trimmed_start = line.len() - line.trim_start().len();
                    format!("{}\\{}", &line[..trimmed_start], &line[trimmed_start..])
                } else {
                    line.to_string()
                }
            })
            .collect();
        lines.join("\n")
    }
}
