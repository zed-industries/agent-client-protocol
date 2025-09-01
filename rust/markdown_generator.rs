use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::fs;
use std::process::Command;

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
            r#"description: "Schema definitions for the Agent Client Protocol""#
        )
        .unwrap();
        writeln!(&mut self.output, "---").unwrap();
        writeln!(&mut self.output).unwrap();

        let mut agent_types: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
        let mut client_types: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
        let mut referenced_types: Vec<(String, Value)> = Vec::new();

        for (name, def) in &self.definitions {
            if def
                .get("x-docs-ignore")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                continue;
            }

            if let Some(side) = def.get("x-side").and_then(|v| v.as_str()) {
                let method = def.get("x-method").unwrap().as_str().unwrap();

                if side == "agent" {
                    agent_types
                        .entry(method.to_string())
                        .or_default()
                        .push((name.to_string(), def.clone()));
                } else {
                    client_types
                        .entry(method.to_string())
                        .or_default()
                        .push((name.to_string(), def.clone()));
                }
            } else {
                referenced_types.push((name.clone(), def.clone()));
            }
        }

        let side_docs = extract_side_docs();

        writeln!(&mut self.output, "## Agent").unwrap();
        writeln!(&mut self.output).unwrap();
        writeln!(&mut self.output, "{}", side_docs.agent_trait).unwrap();
        writeln!(&mut self.output).unwrap();

        for (method, types) in agent_types {
            self.generate_method(&method, side_docs.agent_method_doc(&method), types);
        }

        writeln!(&mut self.output, "## Client").unwrap();
        writeln!(&mut self.output).unwrap();
        writeln!(&mut self.output, "{}", side_docs.client_trait).unwrap();

        for (method, types) in client_types {
            self.generate_method(&method, side_docs.client_method_doc(&method), types);
        }

        referenced_types.sort_by_key(|(name, _)| name.clone());
        for (name, def) in referenced_types {
            self.document_type(2, &name, &def);
        }

        self.output.clone()
    }

    fn generate_method(
        &mut self,
        method: &str,
        docs: &str,
        mut method_types: Vec<(String, Value)>,
    ) {
        if method.contains('/') {
            writeln!(
                &mut self.output,
                "<a id=\"{}\"></a>",
                Self::anchor_text(method).replace("/", "-")
            )
            .unwrap();
        }
        writeln!(
            &mut self.output,
            "### <span class=\"font-mono\">{}</span>",
            method,
        )
        .unwrap();
        writeln!(&mut self.output).unwrap();
        writeln!(&mut self.output, "{}", docs).unwrap();
        writeln!(&mut self.output).unwrap();

        method_types.sort_by_key(|(name, _)| name.clone());

        for (name, def) in method_types {
            self.document_type(4, &name, &def);
        }
    }

    fn document_type(&mut self, headline_level: usize, name: &str, definition: &Value) {
        writeln!(
            &mut self.output,
            "{} <span class=\"font-mono\">{}</span>",
            "#".repeat(headline_level),
            name,
        )
        .unwrap();
        writeln!(&mut self.output).unwrap();

        // Add main description if available
        if let Some(desc) = Self::get_def_description(definition) {
            // Escape # at the beginning of lines to prevent them from being treated as headers
            let escaped_desc = Self::escape_description(&desc);
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
            for variant in variants {
                self.document_variant_table_row(variant);
            }
            writeln!(&mut self.output).unwrap();
        }
    }

    fn document_variant_table_row(&mut self, variant: &Value) {
        write!(&mut self.output, "<ResponseField name=\"").unwrap();

        // Get variant name
        let mut variant_name = String::new();
        if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
            variant_name = type_name.to_string();
            write!(&mut self.output, "{}", type_name).unwrap();
        } else if let Some(const_val) = variant.get("const") {
            if let Some(s) = const_val.as_str() {
                write!(&mut self.output, "{}", s).unwrap();
            } else {
                write!(&mut self.output, "{}", const_val).unwrap();
            }
        } else if variant.get("type").and_then(|v| v.as_str()) == Some("null") {
            write!(&mut self.output, "null").unwrap();
        } else if let Some(props) = variant.get("properties").and_then(|v| v.as_object()) {
            // Look for discriminator
            let discriminator = props
                .iter()
                .find(|(_, v)| v.get("const").is_some())
                .and_then(|(_, v)| v.get("const").and_then(|c| c.as_str()));

            if let Some(const_val) = discriminator {
                write!(&mut self.output, "{}", const_val).unwrap();
            } else {
                write!(&mut self.output, "Object").unwrap();
            }
        } else {
            write!(&mut self.output, "Variant").unwrap();
        }

        writeln!(&mut self.output, "\">").unwrap();

        // Get description
        if let Some(desc) = Self::get_def_description(variant) {
            writeln!(&mut self.output, "{}", desc).unwrap();
        } else {
            writeln!(&mut self.output, "{{\"\"}}").unwrap();
        }

        // Document properties if this variant has them
        if let Some(props) = variant.get("properties").and_then(|v| v.as_object()) {
            if !props.is_empty() {
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "<Expandable title=\"Properties\">").unwrap();
                writeln!(&mut self.output).unwrap();
                self.document_properties_as_fields(props, variant, 0);
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "</Expandable>").unwrap();
            }
        } else if !variant_name.is_empty() {
            // If this is a $ref, look up and document the referenced type's properties
            if let Some(ref_def) = self.definitions.get(&variant_name).cloned()
                && let Some(props) = ref_def.get("properties").and_then(|v| v.as_object())
                && !props.is_empty()
            {
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "<Expandable title=\"Properties\">").unwrap();
                writeln!(&mut self.output).unwrap();
                self.document_properties_as_fields(props, &ref_def, 0);
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "</Expandable>").unwrap();
            }
        }

        writeln!(&mut self.output, "</ResponseField>").unwrap();
        writeln!(&mut self.output).unwrap();
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
            let type_str = Self::get_type_string(prop_schema);

            // Simple field without nesting
            writeln!(
                &mut self.output,
                "{}<ResponseField name=\"{}\" type={{{}}} {}>",
                indent_str,
                prop_name,
                type_str,
                if is_required { "required" } else { "" }
            )
            .unwrap();

            // Add description if available
            if let Some(desc) = Self::get_def_description(prop_schema) {
                writeln!(&mut self.output, "{}  {}", indent_str, desc).unwrap();
            }

            // Add constraints if any
            self.document_field_constraints(prop_schema, indent + 2);

            writeln!(&mut self.output, "{}</ResponseField>", indent_str).unwrap();
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

    fn get_type_string(schema: &Value) -> String {
        // Check for $ref
        if let Some(ref_val) = schema.get("$ref").and_then(|v| v.as_str()) {
            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
            return format!(
                "<a href=\"#{}\">{}</a>",
                Self::anchor_text(type_name),
                type_name
            );
        }

        // Check for type
        if let Some(type_val) = schema.get("type") {
            if let Some(type_str) = type_val.as_str() {
                return match type_str {
                    "array" => {
                        if let Some(items) = schema.get("items") {
                            let item_type = Self::get_type_string(items);
                            format!("<><span>{}</span><span>[]</span></>", item_type)
                        } else {
                            "\"array\"".to_string()
                        }
                    }
                    "integer" => {
                        let type_str =
                            if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
                                format
                            } else {
                                type_str
                            };
                        format!("\"{type_str}\"")
                    }
                    _ => format!("\"{type_str}\""),
                };
            }

            // Handle multiple types (nullable)
            if let Some(arr) = type_val.as_array() {
                let types: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if !types.is_empty() {
                    return format!("\"{}\"", types.join(" | "));
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
                    } else if let Some(t) = Self::get_inline_variant_type(variant) {
                        other_type = Some(t);
                    }
                }
                if has_null && other_type.is_some() {
                    return format!(
                        "<><span>{}</span><span> | null</span></>",
                        other_type.unwrap()
                    );
                }
            }
            return "union".to_string();
        }

        // Check for enum
        if schema.get("enum").is_some() {
            return "\"enum\"".to_string();
        }

        "\"object\"".to_string()
    }

    fn get_inline_variant_type(variant: &Value) -> Option<String> {
        // Check for simple type
        if let Some(type_str) = variant.get("type").and_then(|v| v.as_str()) {
            return Some(format!("\"{type_str}\""));
        }
        // Check for $ref
        if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
            return Some(format!(
                "<a href=\"#{}\">{}</a>",
                Self::anchor_text(type_name),
                type_name
            ));
        }
        None
    }

    fn escape_mdx(text: &str) -> String {
        text.replace('|', "\\|")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('{', "\\{")
            .replace('}', "\\}")
    }

    fn escape_description(text: &str) -> String {
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

    fn get_def_description(def: &Value) -> Option<String> {
        let desc = def
            .get("description")?
            .as_str()?
            .replace("[`", "`")
            .replace("`]", "`");
        let desc = Self::escape_mdx(&desc);
        Some(desc)
    }

    fn anchor_text(title: &str) -> String {
        title.to_lowercase()
    }
}

struct SideDocs {
    agent_trait: String,
    agent_methods: HashMap<String, String>,
    client_trait: String,
    client_methods: HashMap<String, String>,
}

impl SideDocs {
    fn agent_method_doc(&self, method_name: &str) -> &String {
        match method_name {
            "initialize" => self.agent_methods.get("initialize").unwrap(),
            "authenticate" => self.agent_methods.get("authenticate").unwrap(),
            "session/new" => self.agent_methods.get("new_session").unwrap(),
            "session/load" => self.agent_methods.get("load_session").unwrap(),
            "session/prompt" => self.agent_methods.get("prompt").unwrap(),
            "session/cancel" => self.agent_methods.get("cancel").unwrap(),
            "session/list_commands" => self.agent_methods.get("list_commands").unwrap(),
            _ => panic!("Introduced a method? Add it here :)"),
        }
    }

    fn client_method_doc(&self, method_name: &str) -> &String {
        match method_name {
            "session/request_permission" => self.client_methods.get("request_permission").unwrap(),
            "fs/write_text_file" => self.client_methods.get("write_text_file").unwrap(),
            "fs/read_text_file" => self.client_methods.get("read_text_file").unwrap(),
            "session/update" => self.client_methods.get("session_notification").unwrap(),
            "terminal/create" => self.client_methods.get("create_terminal").unwrap(),
            "terminal/output" => self.client_methods.get("terminal_output").unwrap(),
            "terminal/release" => self.client_methods.get("release_terminal").unwrap(),
            "terminal/wait_for_exit" => self.client_methods.get("wait_for_terminal_exit").unwrap(),
            _ => panic!("Introduced a method? Add it here :)"),
        }
    }
}

fn extract_side_docs() -> SideDocs {
    let output = Command::new("cargo")
        .args([
            "+nightly",
            "rustdoc",
            "--lib",
            "--",
            "-Z",
            "unstable-options",
            "--output-format",
            "json",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "Failed to generate rustdoc JSON: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Parse the JSON output
    let json_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/doc/agent_client_protocol.json");
    let json_content = fs::read_to_string(json_path).unwrap();
    let doc: Value = serde_json::from_str(&json_content).unwrap();

    let mut side_docs = SideDocs {
        agent_trait: String::new(),
        agent_methods: HashMap::new(),
        client_trait: String::new(),
        client_methods: HashMap::new(),
    };

    if let Some(index) = doc["index"].as_object() {
        for (_, item) in index {
            if item["name"].as_str() == Some("Agent") {
                if let Some(docs) = item["docs"].as_str() {
                    side_docs.agent_trait = docs.to_string();
                }

                if let Some(items) = item["inner"]["trait"]["items"].as_array() {
                    for method_id in items {
                        if let Some(method) = doc["index"][method_id.to_string()].as_object()
                            && let Some(name) = method["name"].as_str()
                        {
                            side_docs.agent_methods.insert(
                                name.to_string(),
                                method["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }
            }

            if item["name"].as_str() == Some("Client") {
                if let Some(docs) = item["docs"].as_str() {
                    side_docs.client_trait = docs.to_string();
                }

                if let Some(items) = item["inner"]["trait"]["items"].as_array() {
                    for method_id in items {
                        if let Some(method) = doc["index"][method_id.to_string()].as_object()
                            && let Some(name) = method["name"].as_str()
                        {
                            side_docs.client_methods.insert(
                                name.to_string(),
                                method["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }
            }
        }
    }

    side_docs
}
