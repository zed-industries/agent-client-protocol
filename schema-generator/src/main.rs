//! Generates ACP JSON Schema and schema documentation artifacts.

use agent_client_protocol_schema::ProtocolVersion;
#[cfg(not(feature = "unstable_protocol_v2"))]
use agent_client_protocol_schema::v1::{
    AGENT_METHOD_NAMES, AgentNotification, AgentRequest, AgentResponse, CLIENT_METHOD_NAMES,
    ClientNotification, ClientRequest, ClientResponse, JsonRpcMessage, Notification,
    PROTOCOL_LEVEL_METHOD_NAMES, ProtocolLevelNotification, Request, Response,
};
#[cfg(feature = "unstable_protocol_v2")]
use agent_client_protocol_schema::v2::{
    AGENT_METHOD_NAMES, AgentNotification, AgentRequest, AgentResponse, CLIENT_METHOD_NAMES,
    ClientNotification, ClientRequest, ClientResponse, JsonRpcBatch, JsonRpcMessage, Notification,
    PROTOCOL_LEVEL_METHOD_NAMES, ProtocolLevelNotification, Request, Response,
};
use schemars::{
    JsonSchema,
    generate::SchemaSettings,
    transform::{RemoveRefSiblings, ReplaceBoolSchemas},
};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use markdown_generator::MarkdownGenerator;

#[cfg(feature = "unstable_protocol_v2")]
const PROTOCOL_DOC_BASE: &str = "https://agentclientprotocol.com/protocol";

#[cfg(feature = "unstable_protocol_v2")]
const VERSIONED_PROTOCOL_DOC_PATHS: &[&str] = &[
    "agent-plan",
    "authentication",
    "cancellation",
    "content",
    "elicitation",
    "error",
    "extensibility",
    "initialization",
    "prompt-lifecycle",
    "session-config-options",
    "session-delete",
    "session-list",
    "session-setup",
    "slash-commands",
    "tool-calls",
    "transports",
];

/// All messages that an agent can send to a client.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[allow(clippy::large_enum_variant)]
enum AgentOutgoingMessage {
    Request(Request<AgentRequest>),
    Response(Response<AgentResponse>),
    Notification(Notification<AgentNotification>),
}

/// All messages that a client can send to an agent.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[allow(clippy::large_enum_variant)]
enum ClientOutgoingMessage {
    Request(Request<ClientRequest>),
    Response(Response<ClientResponse>),
    Notification(Notification<ClientNotification>),
}

/// Messages that an agent can include in a JSON-RPC batch call to a client.
#[cfg(feature = "unstable_protocol_v2")]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[allow(clippy::large_enum_variant)]
enum AgentBatchCallMessage {
    Request(Request<AgentRequest>),
    Notification(Notification<AgentNotification>),
    ProtocolLevelNotification(Notification<ProtocolLevelNotification>),
}

/// Messages that a client can include in a JSON-RPC batch call to an agent.
#[cfg(feature = "unstable_protocol_v2")]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[allow(clippy::large_enum_variant)]
enum ClientBatchCallMessage {
    Request(Request<ClientRequest>),
    Notification(Notification<ClientNotification>),
    ProtocolLevelNotification(Notification<ProtocolLevelNotification>),
}

#[expect(dead_code)]
#[derive(JsonSchema)]
#[serde(untagged)]
#[schemars(title = "Agent Client Protocol")]
#[allow(clippy::large_enum_variant)]
enum AcpTypes {
    Agent(JsonRpcMessage<AgentOutgoingMessage>),
    Client(JsonRpcMessage<ClientOutgoingMessage>),
    #[cfg(feature = "unstable_protocol_v2")]
    AgentBatchCall(JsonRpcBatch<AgentBatchCallMessage>),
    #[cfg(feature = "unstable_protocol_v2")]
    AgentBatchResponse(JsonRpcBatch<Response<AgentResponse>>),
    #[cfg(feature = "unstable_protocol_v2")]
    ClientBatchCall(JsonRpcBatch<ClientBatchCallMessage>),
    #[cfg(feature = "unstable_protocol_v2")]
    ClientBatchResponse(JsonRpcBatch<Response<ClientResponse>>),
    ProtocolLevel(JsonRpcMessage<Notification<ProtocolLevelNotification>>),
}

fn main() {
    let schema_value = root_schema_value();

    let root = repo_root();
    let schema_dir = root.join("schema");
    let docs_protocol_dir = root.join("docs").join("protocol");

    fs::create_dir_all(schema_dir.clone()).unwrap();
    fs::create_dir_all(docs_protocol_dir.clone()).unwrap();

    write_schema(
        &schema_value,
        schema_dir.as_path(),
        docs_protocol_dir.as_path(),
    );
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("schema-generator manifest directory should have a repository root parent")
        .to_path_buf()
}

#[cfg(test)]
fn schema_crate_dir() -> PathBuf {
    repo_root().join("agent-client-protocol-schema")
}

fn root_schema_value() -> serde_json::Value {
    let mut settings = SchemaSettings::draft2020_12();
    settings.untagged_enum_variant_titles = true;
    let mut bool_schemas = ReplaceBoolSchemas::default();
    bool_schemas.skip_additional_properties = true;
    settings = settings
        .with_transform(RemoveRefSiblings::default())
        .with_transform(bool_schemas);

    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<AcpTypes>();

    // Convert to serde_json::Value for post-processing
    serde_json::to_value(&schema).unwrap()
}

fn write_schema(schema_value: &serde_json::Value, schema_dir: &Path, docs_protocol_dir: &Path) {
    // Each cfg combination owns exactly one filename, with disjoint write
    // sets so the generation runs that produce the published schemas
    // can run in any order without clobbering each other:
    //
    // - `v1/schema.json`           — stable v1 (no features)
    // - `v1/schema.unstable.json`  — v1 + unstable feature flags
    // - `v2/schema.json`           — v2 without unstable feature flags
    // - `v2/schema.unstable.json`  — v2 + unstable feature flags
    let schema_file: &str = match (
        cfg!(feature = "unstable_protocol_v2"),
        cfg!(feature = "unstable"),
    ) {
        (true, true) => "v2/schema.unstable.json",
        (true, false) => "v2/schema.json",
        (false, true) => "v1/schema.unstable.json",
        (false, false) => "v1/schema.json",
    };
    let published_schema_value = schema_value_for_publication(schema_value);
    let schema_json = serde_json::to_string_pretty(&published_schema_value).unwrap();
    let schema_path = schema_dir.join(schema_file);
    if let Some(parent) = schema_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("Failed to create {}: {e}", parent.display()));
    }
    fs::write(schema_path, &schema_json)
        .unwrap_or_else(|e| panic!("Failed to write {schema_file}: {e}"));

    // The version embedded in `meta*.json` reflects the protocol version the
    // *schema itself describes*. Generating with the `unstable_protocol_v2`
    // feature emits v2-shaped types, so the metadata file must advertise
    // version 2 to stay consistent with its contents.
    #[cfg(feature = "unstable_protocol_v2")]
    let schema_protocol_version = ProtocolVersion::V2;
    #[cfg(not(feature = "unstable_protocol_v2"))]
    let schema_protocol_version = ProtocolVersion::V1;

    // Create a combined metadata object
    let metadata = serde_json::json!({
        "version": schema_protocol_version,
        "agentMethods": AGENT_METHOD_NAMES,
        "clientMethods": CLIENT_METHOD_NAMES,
        "protocolMethods": PROTOCOL_LEVEL_METHOD_NAMES,
    });

    let meta_file: &str = match (
        cfg!(feature = "unstable_protocol_v2"),
        cfg!(feature = "unstable"),
    ) {
        (true, true) => "v2/meta.unstable.json",
        (true, false) => "v2/meta.json",
        (false, true) => "v1/meta.unstable.json",
        (false, false) => "v1/meta.json",
    };
    let metadata_json = serde_json::to_string_pretty(&metadata).unwrap();
    let meta_path = schema_dir.join(meta_file);
    if let Some(parent) = meta_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("Failed to create {}: {e}", parent.display()));
    }
    fs::write(meta_path, &metadata_json)
        .unwrap_or_else(|e| panic!("Failed to write {meta_file}: {e}"));

    // Generate markdown documentation. Each cfg combination owns its own
    // doc file, so the `npm run generate` runs don't clobber each other:
    //
    // - `v1/schema.mdx`           — stable v1 (no features)
    // - `v1/draft/schema.mdx`     — v1 + unstable feature flags
    // - `v2/schema.mdx`           — v2 without unstable feature flags
    // - `v2/draft/schema.mdx`     — v2 + unstable feature flags
    let mut markdown_gen = MarkdownGenerator::new(schema_file);
    let mut markdown_doc = markdown_gen.generate(schema_value);

    let protocol_doc_base = match (
        cfg!(feature = "unstable_protocol_v2"),
        cfg!(feature = "unstable"),
    ) {
        (true, true) => "https://agentclientprotocol.com/protocol/v2/draft/",
        (true, false) => "https://agentclientprotocol.com/protocol/v2/",
        (false, true) => "https://agentclientprotocol.com/protocol/v1/draft/",
        (false, false) => "https://agentclientprotocol.com/protocol/v1/",
    };
    markdown_doc = markdown_doc.replace(
        "https://agentclientprotocol.com/protocol/",
        protocol_doc_base,
    );

    let doc_file: &str = match (
        cfg!(feature = "unstable_protocol_v2"),
        cfg!(feature = "unstable"),
    ) {
        (true, true) => "v2/draft/schema.mdx",
        (true, false) => "v2/schema.mdx",
        (false, true) => "v1/draft/schema.mdx",
        (false, false) => "v1/schema.mdx",
    };

    let doc_path = docs_protocol_dir.join(doc_file);
    if let Some(parent) = doc_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("Failed to create {}: {e}", parent.display()));
    }

    fs::write(doc_path, markdown_doc).unwrap_or_else(|e| panic!("Failed to write {doc_file}: {e}"));

    println!("✓ Generated {schema_file}");
    println!("✓ Generated {meta_file}");
    println!("✓ Generated {doc_file}");
}

fn schema_value_for_publication(schema_value: &serde_json::Value) -> serde_json::Value {
    #[cfg(feature = "unstable_protocol_v2")]
    {
        let mut schema_value = schema_value.clone();
        let protocol_doc_base = if cfg!(feature = "unstable") {
            format!("{PROTOCOL_DOC_BASE}/v2/draft")
        } else {
            format!("{PROTOCOL_DOC_BASE}/v2")
        };

        for path in VERSIONED_PROTOCOL_DOC_PATHS {
            replace_string_values(
                &mut schema_value,
                &format!("{PROTOCOL_DOC_BASE}/{path}"),
                &format!("{protocol_doc_base}/{path}"),
            );
        }
        schema_value
    }

    #[cfg(not(feature = "unstable_protocol_v2"))]
    {
        schema_value.clone()
    }
}

#[cfg(feature = "unstable_protocol_v2")]
fn replace_string_values(value: &mut serde_json::Value, from: &str, to: &str) {
    match value {
        serde_json::Value::String(string) => {
            *string = string.replace(from, to);
        }
        serde_json::Value::Array(array) => {
            for value in array {
                replace_string_values(value, from, to);
            }
        }
        serde_json::Value::Object(object) => {
            for value in object.values_mut() {
                replace_string_values(value, from, to);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

#[cfg(test)]
mod schema_annotation_tests {
    #[cfg(feature = "unstable_protocol_v2")]
    use super::{PROTOCOL_DOC_BASE, VERSIONED_PROTOCOL_DOC_PATHS, schema_value_for_publication};
    use super::{root_schema_value, schema_crate_dir};
    use serde_json::Value;
    use std::fs;

    const DEFAULT_ON_ERROR_EXTENSION: &str = "x-deserialize-default-on-error";
    const SKIP_INVALID_ITEMS_EXTENSION: &str = "x-deserialize-skip-invalid-items";

    #[test]
    fn generated_schema_includes_tolerant_deserialization_extensions() {
        let schema = root_schema_value();

        #[cfg(not(feature = "unstable_protocol_v2"))]
        {
            let client_info = property_schema(&schema, "InitializeRequest", "clientInfo");
            assert_bool_extension(client_info, DEFAULT_ON_ERROR_EXTENSION);
            assert_no_extension(client_info, SKIP_INVALID_ITEMS_EXTENSION);
        }

        #[cfg(feature = "unstable_protocol_v2")]
        {
            let request = def_schema(&schema, "InitializeRequest");
            assert!(
                request
                    .pointer("/required")
                    .and_then(Value::as_array)
                    .is_some_and(|required| required.iter().any(|field| field == "info"))
            );
            let info = property_schema(&schema, "InitializeRequest", "info");
            assert_no_extension(info, DEFAULT_ON_ERROR_EXTENSION);
            assert_no_extension(info, SKIP_INVALID_ITEMS_EXTENSION);

            let response = def_schema(&schema, "InitializeResponse");
            assert!(
                response
                    .pointer("/required")
                    .and_then(Value::as_array)
                    .is_some_and(|required| required.iter().any(|field| field == "info"))
            );
        }

        let auth_methods = property_schema(&schema, "InitializeResponse", "authMethods");
        assert_bool_extension(auth_methods, DEFAULT_ON_ERROR_EXTENSION);
        assert_bool_extension(auth_methods, SKIP_INVALID_ITEMS_EXTENSION);
    }

    #[cfg(feature = "unstable_protocol_v2")]
    #[test]
    fn generated_v2_schema_includes_json_rpc_batch_messages() {
        let schema = root_schema_value();
        for title in [
            "AgentBatchCall",
            "AgentBatchResponse",
            "ClientBatchCall",
            "ClientBatchResponse",
        ] {
            let batch_schema = root_variant_schema(&schema, title);
            assert_eq!(
                batch_schema.get("type").and_then(Value::as_str),
                Some("array")
            );
            assert_eq!(
                batch_schema.get("minItems").and_then(Value::as_u64),
                Some(1)
            );
        }

        for title in ["AgentBatchCall", "ClientBatchCall"] {
            let batch_schema = root_variant_schema(&schema, title);
            assert!(
                schema_contains_ref(batch_schema, "#/$defs/ProtocolLevelNotification"),
                "missing ProtocolLevelNotification in {title}"
            );
        }

        {
            let protocol_level = root_variant_schema(&schema, "ProtocolLevel");
            assert_eq!(
                protocol_level
                    .pointer("/properties/jsonrpc/enum/0")
                    .and_then(Value::as_str),
                Some("2.0")
            );
            assert_eq!(
                protocol_level
                    .pointer("/properties/method/type")
                    .and_then(Value::as_str),
                Some("string")
            );

            let protocol_notification = def_schema(&schema, "ProtocolLevelNotification");
            assert_eq!(
                protocol_notification
                    .pointer("/properties/method/type")
                    .and_then(Value::as_str),
                Some("string")
            );
            assert!(
                protocol_notification
                    .pointer("/required")
                    .and_then(Value::as_array)
                    .is_some_and(|required| required.iter().any(|field| field == "method"))
            );
            assert!(
                schema_contains_ref(protocol_notification, "#/$defs/CancelRequestNotification"),
                "missing CancelRequestNotification in ProtocolLevelNotification"
            );
        }
    }

    #[cfg(feature = "unstable_protocol_v2")]
    #[test]
    fn generated_v2_schema_uses_standard_base64_content_encoding() {
        let schema = root_schema_value();

        for (definition, property) in [
            ("ImageContent", "data"),
            ("AudioContent", "data"),
            ("BlobResourceContents", "blob"),
            ("TerminalOutput", "data"),
            ("TerminalOutputChunk", "data"),
        ] {
            let property = property_schema(&schema, definition, property);
            assert_eq!(
                property.get("contentEncoding").and_then(Value::as_str),
                Some("base64"),
                "missing base64 content encoding on {definition}"
            );
            assert!(
                property.get("format").is_none(),
                "legacy byte format remains on {definition}"
            );
        }
    }

    #[cfg(feature = "unstable_protocol_v2")]
    #[test]
    fn generated_v2_schema_references_semantic_string_types() {
        let schema = root_schema_value();

        for definition in ["AbsolutePath", "SessionListCursor", "MediaType"] {
            assert_eq!(
                def_schema(&schema, definition)
                    .get("type")
                    .and_then(Value::as_str),
                Some("string"),
                "{definition} must remain wire-compatible with a JSON string"
            );
        }

        for (definition, property, reference) in [
            ("NewSessionRequest", "cwd", "AbsolutePath"),
            ("NewSessionRequest", "additionalDirectories", "AbsolutePath"),
            ("ResumeSessionRequest", "cwd", "AbsolutePath"),
            (
                "ResumeSessionRequest",
                "additionalDirectories",
                "AbsolutePath",
            ),
            ("ListSessionsRequest", "cwd", "AbsolutePath"),
            ("SessionInfo", "cwd", "AbsolutePath"),
            ("SessionInfo", "additionalDirectories", "AbsolutePath"),
            ("McpServerStdio", "command", "AbsolutePath"),
            ("CommandPermissionSubject", "cwd", "AbsolutePath"),
            ("TerminalUpdate", "cwd", "AbsolutePath"),
            ("DiffPathChange", "path", "AbsolutePath"),
            ("DiffPathPairChange", "oldPath", "AbsolutePath"),
            ("DiffPathPairChange", "path", "AbsolutePath"),
            ("ToolCallLocation", "path", "AbsolutePath"),
            ("ListSessionsRequest", "cursor", "SessionListCursor"),
            ("ListSessionsResponse", "nextCursor", "SessionListCursor"),
            ("ImageContent", "mimeType", "MediaType"),
            ("AudioContent", "mimeType", "MediaType"),
            ("TextResourceContents", "mimeType", "MediaType"),
            ("BlobResourceContents", "mimeType", "MediaType"),
            ("ResourceLink", "mimeType", "MediaType"),
            ("Icon", "mimeType", "MediaType"),
            ("DiffChange", "mimeType", "MediaType"),
        ] {
            let property = property_schema(&schema, definition, property);
            assert!(
                schema_contains_ref(property, &format!("#/$defs/{reference}")),
                "missing {reference} reference on {definition}"
            );
        }

        #[cfg(feature = "unstable")]
        for property in ["cwd", "additionalDirectories"] {
            assert!(schema_contains_ref(
                property_schema(&schema, "ForkSessionRequest", property),
                "#/$defs/AbsolutePath"
            ));
        }
    }

    #[cfg(feature = "unstable_protocol_v2")]
    #[test]
    fn published_v2_schema_links_to_versioned_v2_protocol_docs() {
        let schema = schema_value_for_publication(&root_schema_value());
        let schema_json = serde_json::to_string(&schema).unwrap();

        let protocol_doc_base = if cfg!(feature = "unstable") {
            format!("{PROTOCOL_DOC_BASE}/v2/draft")
        } else {
            format!("{PROTOCOL_DOC_BASE}/v2")
        };

        assert!(schema_json.contains(&format!("{protocol_doc_base}/prompt-lifecycle")));
        assert!(schema_json.contains(&format!("{protocol_doc_base}/tool-calls")));
        assert!(schema_json.contains(&format!("{protocol_doc_base}/initialization")));
        assert!(schema_json.contains(&format!("{protocol_doc_base}/elicitation")));

        for path in VERSIONED_PROTOCOL_DOC_PATHS {
            assert!(
                !schema_json.contains(&format!("{PROTOCOL_DOC_BASE}/{path}")),
                "found unversioned v2 schema docs link for {path}"
            );
        }

        assert!(
            !schema_json.contains(&format!("{PROTOCOL_DOC_BASE}/v2/v2/")),
            "v2 docs links should not be double-versioned"
        );
    }

    #[test]
    fn source_default_on_error_fields_are_schema_annotated() {
        let root = schema_crate_dir();
        for module_dir in ["src/v1", "src/v2"] {
            for entry in fs::read_dir(root.join(module_dir)).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                    continue;
                }

                let source = fs::read_to_string(&path).unwrap();
                let lines: Vec<_> = source.lines().collect();
                for (line_index, line) in lines.iter().enumerate() {
                    if !line.contains(r#"#[serde_as(deserialize_as = "DefaultOnError"#) {
                        continue;
                    }

                    let annotation = lines.get(line_index + 1).copied().unwrap_or_default();
                    assert!(
                        annotation.contains(r#""x-deserialize-default-on-error" = true"#),
                        "{}:{} missing {DEFAULT_ON_ERROR_EXTENSION}",
                        path.display(),
                        line_index + 1
                    );

                    if line.contains("VecSkipError") {
                        assert!(
                            annotation.contains(r#""x-deserialize-skip-invalid-items" = true"#),
                            "{}:{} missing {SKIP_INVALID_ITEMS_EXTENSION}",
                            path.display(),
                            line_index + 1
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn source_meta_fields_are_default_on_error_annotated() {
        let root = schema_crate_dir();
        for module_dir in ["src/v1", "src/v2"] {
            for entry in fs::read_dir(root.join(module_dir)).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                    continue;
                }

                let source = fs::read_to_string(&path).unwrap();
                let lines: Vec<_> = source.lines().collect();
                for (line_index, line) in lines.iter().enumerate() {
                    if !line.trim_start().starts_with("pub meta:") || !line.contains("Meta") {
                        continue;
                    }

                    let annotations = lines
                        .get(line_index.saturating_sub(8)..line_index)
                        .unwrap_or_default()
                        .join("\n");

                    assert!(
                        annotations.contains(r#"#[serde_as(deserialize_as = "DefaultOnError"#),
                        "{}:{} missing DefaultOnError on meta field",
                        path.display(),
                        line_index + 1
                    );
                    assert!(
                        annotations.contains(r#""x-deserialize-default-on-error" = true"#),
                        "{}:{} missing {DEFAULT_ON_ERROR_EXTENSION} on meta field",
                        path.display(),
                        line_index + 1
                    );
                }
            }
        }
    }

    #[test]
    fn generated_schema_meta_fields_are_default_on_error_annotated() {
        let schema = root_schema_value();
        let mut checked = 0;

        assert_meta_properties_are_annotated(&schema, &mut checked);

        assert!(checked > 0, "expected at least one _meta schema property");
    }

    fn property_schema<'a>(schema: &'a Value, def_name: &str, prop_name: &str) -> &'a Value {
        def_schema(schema, def_name)
            .pointer(&format!("/properties/{prop_name}"))
            .unwrap_or_else(|| panic!("missing schema property {def_name}.{prop_name}"))
    }

    fn def_schema<'a>(schema: &'a Value, def_name: &str) -> &'a Value {
        schema
            .pointer(&format!("/$defs/{def_name}"))
            .unwrap_or_else(|| panic!("missing schema definition {def_name}"))
    }

    #[cfg(feature = "unstable_protocol_v2")]
    fn root_variant_schema<'a>(schema: &'a Value, title: &str) -> &'a Value {
        schema
            .get("anyOf")
            .and_then(Value::as_array)
            .and_then(|variants| {
                variants
                    .iter()
                    .find(|variant| variant.get("title").and_then(Value::as_str) == Some(title))
            })
            .unwrap_or_else(|| panic!("missing root schema variant {title}"))
    }

    fn assert_bool_extension(schema: &Value, extension: &str) {
        assert_eq!(
            schema.get(extension).and_then(Value::as_bool),
            Some(true),
            "missing extension {extension} on {schema}"
        );
    }

    fn assert_meta_properties_are_annotated(schema: &Value, checked: &mut usize) {
        match schema {
            Value::Object(object) => {
                if let Some(properties) = object.get("properties").and_then(Value::as_object)
                    && let Some(meta) = properties.get("_meta")
                {
                    *checked += 1;
                    assert_bool_extension(meta, DEFAULT_ON_ERROR_EXTENSION);
                }

                for value in object.values() {
                    assert_meta_properties_are_annotated(value, checked);
                }
            }
            Value::Array(values) => {
                for value in values {
                    assert_meta_properties_are_annotated(value, checked);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }

    fn assert_no_extension(schema: &Value, extension: &str) {
        assert!(
            schema.get(extension).is_none(),
            "unexpected extension {extension} on {schema}"
        );
    }

    #[cfg(feature = "unstable_protocol_v2")]
    fn schema_contains_ref(schema: &Value, ref_path: &str) -> bool {
        match schema {
            Value::Object(object) => object.iter().any(|(key, value)| {
                key == "$ref" && value.as_str() == Some(ref_path)
                    || schema_contains_ref(value, ref_path)
            }),
            Value::Array(array) => array
                .iter()
                .any(|value| schema_contains_ref(value, ref_path)),
            _ => false,
        }
    }
}

mod markdown_generator {
    use serde_json::Value;
    use std::collections::{BTreeMap, BTreeSet, HashMap};
    use std::fmt::Write;
    use std::fs;
    use std::process::Command;

    pub struct MarkdownGenerator {
        definitions: BTreeMap<String, Value>,
        output: String,
        schema_file: &'static str,
    }

    impl MarkdownGenerator {
        pub fn new(schema_file: &'static str) -> Self {
            Self {
                definitions: BTreeMap::new(),
                output: String::new(),
                schema_file,
            }
        }

        #[expect(clippy::too_many_lines)]
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
            let schema_file = self.schema_file;
            if schema_file.starts_with("v2/") {
                writeln!(
                    &mut self.output,
                    "<Note>This schema file is generated in this repository at [`schema/{schema_file}`](https://github.com/agentclientprotocol/agent-client-protocol/blob/main/schema/{schema_file}). GitHub releases for this schema are not published yet.</Note>"
                )
                .unwrap();
            } else {
                let download_file = schema_file.rsplit('/').next().unwrap_or(schema_file);
                writeln!(
                    &mut self.output,
                    "<Note>The schema file can be downloaded directly from the [latest GitHub release](https://github.com/agentclientprotocol/agent-client-protocol/releases/latest/download/{download_file}).</Note>"
                )
                .unwrap();
            }
            writeln!(&mut self.output).unwrap();

            let mut agent_types: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
            let mut client_types: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
            let mut protocol_types: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
            let mut referenced_types: Vec<(String, Value)> = Vec::new();

            for (name, def) in &self.definitions {
                if def
                    .get("x-docs-ignore")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                {
                    continue;
                }

                if let Some(side) = def.get("x-side").and_then(|v| v.as_str()) {
                    let method = def.get("x-method").unwrap().as_str().unwrap();

                    let types = match side {
                        "agent" => &mut agent_types,
                        "client" => &mut client_types,
                        "protocol" => &mut protocol_types,
                        "both" => {
                            let entry = (name.clone(), def.clone());
                            agent_types
                                .entry(method.to_string())
                                .or_default()
                                .push(entry.clone());
                            client_types
                                .entry(method.to_string())
                                .or_default()
                                .push(entry);
                            continue;
                        }
                        _ => unimplemented!("Unexpected side {side}"),
                    };

                    types
                        .entry(method.to_string())
                        .or_default()
                        .push((name.clone(), def.clone()));
                } else {
                    referenced_types.push((name.clone(), def.clone()));
                }
            }

            let side_docs = extract_side_docs();
            let mut duplicate_methods = BTreeSet::new();
            for method in agent_types.keys() {
                if client_types.contains_key(method) || protocol_types.contains_key(method) {
                    duplicate_methods.insert(method.clone());
                }
            }
            for method in client_types.keys() {
                if protocol_types.contains_key(method) {
                    duplicate_methods.insert(method.clone());
                }
            }

            writeln!(&mut self.output, "## Agent").unwrap();
            writeln!(&mut self.output).unwrap();
            writeln!(
                &mut self.output,
                "Defines the interface that all ACP-compliant agents must implement.

Agents are programs that use generative AI to autonomously modify code. They handle
requests from clients and execute tasks using language models and tools."
            )
            .unwrap();
            writeln!(&mut self.output).unwrap();

            for (method, types) in agent_types {
                let anchor_prefix = duplicate_methods.contains(&method).then_some("agent");
                self.generate_method(
                    anchor_prefix,
                    &method,
                    side_docs.agent_method_doc(&method),
                    types,
                );
            }

            writeln!(&mut self.output, "## Client").unwrap();
            writeln!(&mut self.output).unwrap();
            writeln!(
                &mut self.output,
                "Defines the interface that ACP-compliant clients must implement.

Clients are typically code editors (IDEs, text editors) that provide the interface
between users and AI agents. They manage the environment, handle user interactions,
and control access to resources."
            )
            .unwrap();

            for (method, types) in client_types {
                let anchor_prefix = duplicate_methods.contains(&method).then_some("client");
                self.generate_method(
                    anchor_prefix,
                    &method,
                    side_docs.client_method_doc(&method),
                    types,
                );
            }
            {
                writeln!(&mut self.output, "## Protocol Level").unwrap();
                writeln!(&mut self.output).unwrap();
                writeln!(
            &mut self.output,
            "Defines the interface that ACP-compliant agents and clients must both implement.

Notifications whose methods start with '$/' are messages which are protocol
implementation dependent and might not be implementable in all clients or
agents. For example if the implementation uses a single threaded synchronous
programming language then there is little it can do to react to a
`$/cancel_request` notification. If an agent or client receives notifications
starting with '$/' it is free to ignore the notification."
        )
                .unwrap();

                for (method, types) in protocol_types {
                    let anchor_prefix = duplicate_methods.contains(&method).then_some("protocol");
                    self.generate_method(
                        anchor_prefix,
                        &method,
                        side_docs.protocol_method_doc(&method),
                        types,
                    );
                }
            }

            referenced_types.sort_by_key(|(name, _)| name.clone());
            for (name, def) in referenced_types {
                self.document_type(2, &name, &def);
            }

            self.output.clone()
        }

        fn generate_method(
            &mut self,
            anchor_prefix: Option<&str>,
            method: &str,
            docs: &str,
            mut method_types: Vec<(String, Value)>,
        ) {
            if method.contains('/') {
                let mut anchor = Self::anchor_text(method).replace('/', "-");
                if let Some(prefix) = anchor_prefix {
                    anchor = format!("{prefix}-{anchor}");
                }
                writeln!(&mut self.output, "<a id=\"{anchor}\"></a>").unwrap();
            }
            writeln!(
                &mut self.output,
                "### <span class=\"font-mono\">{method}</span>",
            )
            .unwrap();
            writeln!(&mut self.output).unwrap();
            writeln!(&mut self.output, "{docs}").unwrap();
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
                writeln!(&mut self.output, "{escaped_desc}").unwrap();
                writeln!(&mut self.output).unwrap();
            }
            // Determine type kind and document accordingly
            if let Some(variants) = definition
                .get("oneOf")
                .or_else(|| definition.get("anyOf"))
                .and_then(|v| v.as_array())
            {
                if variants.len() == 1 {
                    // Single-variant union: resolve the $ref and render as its
                    // underlying type instead of a "Union" wrapper.
                    let variant = &variants[0];
                    if let Some(merged_def) = self.merge_variant_definition(variant) {
                        // Preserve variant-level description if present
                        if let Some(desc) = Self::get_def_description(variant) {
                            let escaped_desc = Self::escape_description(&desc);
                            writeln!(&mut self.output, "{escaped_desc}").unwrap();
                            writeln!(&mut self.output).unwrap();
                        }
                        if merged_def.get("properties").is_some() {
                            self.document_object(&merged_def);
                        } else if let Some(type_val) =
                            merged_def.get("type").and_then(|v| v.as_str())
                        {
                            self.document_simple_type(type_val, &merged_def);
                        } else {
                            self.document_union(definition);
                        }
                    } else {
                        self.document_union(definition);
                    }
                } else {
                    self.document_union(definition);
                }
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

            let discriminator_prop = definition
                .get("discriminator")
                .and_then(|d| d.get("propertyName"))
                .and_then(|p| p.as_str());

            let any_of = definition.get("anyOf").and_then(|v| v.as_array());
            let one_of = definition.get("oneOf").and_then(|v| v.as_array());

            // Union types with top-level "properties" alongside "oneOf"/"anyOf" use them
            // as shared properties that apply to all variants (e.g., _meta, message).
            // The discriminator property (if any) is excluded since it's per-variant.
            let has_shared_props = if let Some(shared_props) =
                definition.get("properties").and_then(|v| v.as_object())
            {
                let filtered_props: serde_json::Map<String, Value> = shared_props
                    .iter()
                    .filter(|(key, _)| Some(key.as_str()) != discriminator_prop)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                if filtered_props.is_empty() {
                    false
                } else {
                    writeln!(&mut self.output, "**Shared properties:**").unwrap();
                    writeln!(&mut self.output).unwrap();
                    self.document_properties_as_fields(&filtered_props, definition, 0);
                    writeln!(&mut self.output).unwrap();
                    true
                }
            } else {
                false
            };

            // Print a single "Variants:" label before all variant groups when
            // there is surrounding context that benefits from a separator
            // (shared properties above, or multiple variant groups).
            if has_shared_props || (any_of.is_some() && one_of.is_some()) {
                writeln!(&mut self.output, "**Variants:**").unwrap();
                writeln!(&mut self.output).unwrap();
            }

            if let Some(variants) = any_of {
                for variant in variants {
                    self.document_variant_table_row(variant);
                }
                writeln!(&mut self.output).unwrap();
            }

            if let Some(variants) = one_of {
                for variant in variants {
                    self.document_variant_table_row(variant);
                }
                writeln!(&mut self.output).unwrap();
            }
        }

        #[expect(clippy::too_many_lines)]
        fn document_variant_table_row(&mut self, variant: &Value) {
            let enum_values = variant.get("enum").and_then(|v| v.as_array());

            write!(&mut self.output, "<ResponseField name=\"").unwrap();

            // Get variant name
            if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
                let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                write!(&mut self.output, "{type_name}").unwrap();
            } else if let Some(enum_values) = enum_values {
                write!(&mut self.output, "{}", Self::enum_values_label(enum_values)).unwrap();
            } else if let Some(const_val) = variant.get("const") {
                if let Some(s) = const_val.as_str() {
                    write!(&mut self.output, "{s}").unwrap();
                } else {
                    write!(&mut self.output, "{const_val}").unwrap();
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
                    write!(&mut self.output, "{const_val}").unwrap();
                } else if let Some(title) = variant.get("title").and_then(|t| t.as_str()) {
                    write!(&mut self.output, "{title}").unwrap();
                } else {
                    write!(&mut self.output, "Object").unwrap();
                }
            } else if let Some(title) = variant.get("title") {
                if let Some(s) = title.as_str() {
                    write!(&mut self.output, "{s}").unwrap();
                } else {
                    write!(&mut self.output, "{title}").unwrap();
                }
            } else if let Some(ty) = variant.get("type") {
                if let Some(s) = ty.as_str() {
                    write!(&mut self.output, "{s}").unwrap();
                } else {
                    write!(&mut self.output, "{ty}").unwrap();
                }
            } else {
                write!(&mut self.output, "Variant").unwrap();
            }

            if enum_values.is_some() {
                write!(&mut self.output, "\" type=\"enum").unwrap();
            } else if let Some(format) = variant.get("format") {
                if let Some(s) = format.as_str() {
                    write!(&mut self.output, "\" type=\"{s}").unwrap();
                } else {
                    write!(&mut self.output, "\" type=\"{format}").unwrap();
                }
            } else if let Some(ty) = variant.get("type") {
                if let Some(s) = ty.as_str() {
                    write!(&mut self.output, "\" type=\"{s}").unwrap();
                } else {
                    write!(&mut self.output, "\" type=\"{ty}").unwrap();
                }
            }

            writeln!(&mut self.output, "\">").unwrap();

            // Get description
            if let Some(desc) = Self::get_def_description(variant) {
                writeln!(&mut self.output, "{desc}").unwrap();
            }

            if let Some(enum_values) = enum_values {
                if Self::get_def_description(variant).is_some() {
                    writeln!(&mut self.output).unwrap();
                }
                writeln!(
                    &mut self.output,
                    "**Values:** {}",
                    Self::format_enum_values(enum_values)
                )
                .unwrap();
            } else if Self::get_def_description(variant).is_none() {
                writeln!(&mut self.output, "{{\"\"}}").unwrap();
            }

            // Collect all properties and required fields
            let mut merged_props = serde_json::Map::new();
            let mut merged_required = Vec::new();

            // Helper to merge from a definition
            let mut merge_from = |def: &Value| {
                if let Some(props) = def.get("properties").and_then(|v| v.as_object()) {
                    for (k, v) in props {
                        merged_props.insert(k.clone(), v.clone());
                    }
                }
                if let Some(req) = def.get("required").and_then(|v| v.as_array()) {
                    for r in req {
                        if !merged_required.contains(r) {
                            merged_required.push(r.clone());
                        }
                    }
                }
            };

            // 1. Check for $ref (direct)
            if let Some(merged_variant) = self.merge_variant_definition(variant) {
                merge_from(&merged_variant);
            } else {
                // 1. Check for $ref (direct)
                if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
                    let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                    if let Some(ref_def) = self.definitions.get(type_name) {
                        merge_from(ref_def);
                    }
                }

                // 2. Check for allOf (often used for inheritance/composition)
                if let Some(all_of) = variant.get("allOf").and_then(|v| v.as_array()) {
                    for item in all_of {
                        if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                            if let Some(ref_def) = self.definitions.get(type_name) {
                                merge_from(ref_def);
                            }
                        } else {
                            merge_from(item);
                        }
                    }
                }

                // 3. Local properties
                merge_from(variant);
            }

            if !merged_props.is_empty() {
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "<Expandable title=\"Properties\">").unwrap();
                writeln!(&mut self.output).unwrap();

                let mut synthetic_def = serde_json::Map::new();
                synthetic_def.insert("required".to_string(), Value::Array(merged_required));

                self.document_properties_as_fields(&merged_props, &Value::Object(synthetic_def), 0);
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "</Expandable>").unwrap();
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
                        write!(&mut self.output, "`\"{s}\"`").unwrap();
                    } else {
                        write!(&mut self.output, "`{val}`").unwrap();
                    }
                    writeln!(&mut self.output, " |").unwrap();
                }
                writeln!(&mut self.output).unwrap();
            }
        }

        fn enum_values_label(values: &[Value]) -> String {
            values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map_or_else(|| value.to_string(), str::to_string)
                })
                .collect::<Vec<_>>()
                .join(" | ")
        }

        fn format_enum_values(values: &[Value]) -> String {
            values
                .iter()
                .map(|value| {
                    if let Some(value) = value.as_str() {
                        format!("`\"{}\"`", Self::escape_mdx(value))
                    } else {
                        format!("`{value}`")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
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
                    writeln!(&mut self.output, "{indent_str}  {desc}").unwrap();
                } else if let Some(const_val) = prop_schema.get("const") {
                    let val_str = if let Some(s) = const_val.as_str() {
                        format!("\"{s}\"")
                    } else {
                        const_val.to_string()
                    };
                    writeln!(
                        &mut self.output,
                        "{indent_str}  The discriminator value. Must be `{val_str}`."
                    )
                    .unwrap();
                }

                // Add constraints if any
                self.document_field_constraints(prop_schema, indent + 2);

                writeln!(&mut self.output, "{indent_str}</ResponseField>").unwrap();
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
                constraints.push(("Minimum", format!("`{v}`")));
            }
            if let Some(v) = schema.get("maximum") {
                constraints.push(("Maximum", format!("`{v}`")));
            }
            if let Some(v) = schema.get("minLength") {
                constraints.push(("Min length", format!("`{v}`")));
            }
            if let Some(v) = schema.get("maxLength") {
                constraints.push(("Max length", format!("`{v}`")));
            }
            if let Some(v) = schema.get("pattern") {
                constraints.push(("Pattern", format!("`{v}`")));
            }
            Self::append_string_annotations(schema, &mut constraints);

            if !constraints.is_empty() {
                writeln!(&mut self.output).unwrap();
                if constraints.len() == 1 {
                    // Single constraint as text
                    let (name, value) = &constraints[0];
                    writeln!(&mut self.output, "{indent_str}  - {name}: {value}").unwrap();
                } else {
                    // Multiple constraints as table
                    writeln!(&mut self.output, "{indent_str}  | Constraint | Value |").unwrap();
                    writeln!(&mut self.output, "{indent_str}  | ---------- | ----- |").unwrap();
                    for (name, value) in constraints {
                        writeln!(&mut self.output, "{indent_str}  | {name} | {value} |").unwrap();
                    }
                }
            }

            // Document enum values if present
            if let Some(enum_vals) = schema.get("enum").and_then(|v| v.as_array()) {
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "{indent_str}  **Allowed values:**").unwrap();
                for val in enum_vals {
                    if let Some(s) = val.as_str() {
                        writeln!(&mut self.output, "{indent_str}  - `\"{s}\"`").unwrap();
                    } else {
                        writeln!(&mut self.output, "{indent_str}  - `{val}`").unwrap();
                    }
                }
            }
        }

        fn document_simple_type(&mut self, type_name: &str, definition: &Value) {
            let formatted_type = match type_name {
                "integer" => {
                    if let Some(format) = definition.get("format").and_then(|v| v.as_str()) {
                        format!("integer ({format})")
                    } else {
                        "integer".to_string()
                    }
                }
                "number" => {
                    if let Some(format) = definition.get("format").and_then(|v| v.as_str()) {
                        format!("number ({format})")
                    } else {
                        "number".to_string()
                    }
                }
                "string" => {
                    if let Some(format) = definition.get("format").and_then(|v| v.as_str()) {
                        format!("string ({format})")
                    } else {
                        "string".to_string()
                    }
                }
                _ => type_name.to_string(),
            };

            writeln!(&mut self.output, "**Type:** `{formatted_type}`").unwrap();

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
                constraints.push(("Minimum", format!("`{v}`")));
            }
            if let Some(v) = schema.get("maximum") {
                constraints.push(("Maximum", format!("`{v}`")));
            }
            if let Some(v) = schema.get("minLength") {
                constraints.push(("Min length", format!("`{v}`")));
            }
            if let Some(v) = schema.get("maxLength") {
                constraints.push(("Max length", format!("`{v}`")));
            }
            if let Some(v) = schema.get("pattern") {
                constraints.push(("Pattern", format!("`{v}`")));
            }
            Self::append_string_annotations(schema, &mut constraints);

            if !constraints.is_empty() {
                writeln!(&mut self.output).unwrap();
                if constraints.len() == 1 {
                    // Single constraint as text
                    let (name, value) = &constraints[0];
                    writeln!(&mut self.output, "**{name}:** {value}").unwrap();
                } else {
                    // Multiple constraints as table
                    writeln!(&mut self.output, "| Constraint | Value |").unwrap();
                    writeln!(&mut self.output, "| ---------- | ----- |").unwrap();
                    for (name, value) in constraints {
                        writeln!(&mut self.output, "| {name} | {value} |").unwrap();
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
                        write!(&mut self.output, "`\"{s}\"`").unwrap();
                    } else {
                        write!(&mut self.output, "`{val}`").unwrap();
                    }
                    writeln!(&mut self.output, " |").unwrap();
                }
            }
        }

        fn append_string_annotations(
            schema: &Value,
            constraints: &mut Vec<(&'static str, String)>,
        ) {
            if !Self::is_string_capable(schema) {
                return;
            }

            for (keyword, label) in [
                ("format", "Format"),
                ("contentEncoding", "Content encoding"),
                ("contentMediaType", "Content media type"),
            ] {
                if let Some(value) = schema.get(keyword).and_then(Value::as_str) {
                    constraints.push((label, format!("`{value}`")));
                }
            }
        }

        fn is_string_capable(schema: &Value) -> bool {
            match schema.get("type") {
                Some(Value::String(schema_type)) => schema_type == "string",
                Some(Value::Array(schema_types)) => schema_types
                    .iter()
                    .any(|schema_type| schema_type.as_str() == Some("string")),
                _ => false,
            }
        }

        fn get_ref_type_name(schema: &Value) -> Option<&str> {
            if let Some(ref_val) = schema.get("$ref").and_then(|v| v.as_str()) {
                return Some(ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val));
            }

            // Check for single-item allOf/anyOf/oneOf wrappers (often used for $ref with sibling properties)
            for key in ["allOf", "anyOf", "oneOf"] {
                if let Some(arr) = schema.get(key).and_then(|v| v.as_array())
                    && arr.len() == 1
                    && let Some(type_name) = Self::get_ref_type_name(&arr[0])
                {
                    return Some(type_name);
                }
            }

            None
        }

        fn get_array_type_string(schema: &Value) -> String {
            if let Some(items) = schema.get("items") {
                if let Some(type_name) = Self::get_ref_type_name(items) {
                    return format!(
                        "<a href=\"#{}\">{}[]</a>",
                        MarkdownGenerator::anchor_text(type_name),
                        type_name
                    );
                }

                let item_type = MarkdownGenerator::get_type_string(items);
                format!("<><span>{item_type}</span><span>[]</span></>")
            } else {
                "\"array\"".to_string()
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

            // Check for single-item allOf/anyOf/oneOf wrappers (often used for $ref with sibling properties)
            for key in ["allOf", "anyOf", "oneOf"] {
                if let Some(arr) = schema.get(key).and_then(|v| v.as_array())
                    && arr.len() == 1
                {
                    return Self::get_type_string(&arr[0]);
                }
            }

            // Check for type
            if let Some(type_val) = schema.get("type") {
                if let Some(type_str) = type_val.as_str() {
                    return match type_str {
                        "array" => Self::get_array_type_string(schema),
                        "integer" => {
                            let type_str = if let Some(format) =
                                schema.get("format").and_then(|v| v.as_str())
                            {
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
                    let types: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                    if types.is_empty() {
                        return "\"object\"".to_string();
                    }

                    // Special-case nullable arrays so we can still show the item type (and link to it).
                    if types.contains(&"array") && schema.get("items").is_some() {
                        let array_type = Self::get_array_type_string(schema);
                        let rest: Vec<&str> =
                            types.iter().copied().filter(|t| *t != "array").collect();
                        if rest.is_empty() {
                            return array_type;
                        }
                        let rest_text = rest.join(" | ");
                        return format!(
                            "<><span>{array_type}</span><span> | {rest_text}</span></>"
                        );
                    }

                    return format!("\"{}\"", types.join(" | "));
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
                    if has_null && let Some(other_type) = other_type {
                        return format!("<><span>{other_type}</span><span> | null</span></>");
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
            if variant.get("oneOf").is_some() || variant.get("anyOf").is_some() {
                return None;
            }

            // Check for simple type
            if variant.get("type").and_then(|v| v.as_str()).is_some() {
                return Some(Self::get_type_string(variant));
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

        fn merge_variant_definition(&self, variant: &Value) -> Option<Value> {
            let mut merged = if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
                let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                self.definitions.get(type_name).cloned()?
            } else {
                let all_of = variant.get("allOf").and_then(|v| v.as_array())?;
                let mut base = None;

                for item in all_of {
                    if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                        let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                        if let Some(def) = self.definitions.get(type_name) {
                            base = Some(def.clone());
                            break;
                        }
                    }
                }

                base.unwrap_or_else(|| Value::Object(serde_json::Map::new()))
            };

            let Some(merged_obj) = merged.as_object_mut() else {
                return Some(merged);
            };

            let mut wrapper_props = serde_json::Map::new();
            let mut wrapper_required = Vec::new();

            let mut collect_fields = |schema: &Value| {
                if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
                    for (key, value) in props {
                        wrapper_props
                            .entry(key.clone())
                            .or_insert_with(|| value.clone());
                    }
                }
                if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
                    for req in required {
                        if !wrapper_required.contains(req) {
                            wrapper_required.push(req.clone());
                        }
                    }
                }
            };

            if let Some(all_of) = variant.get("allOf").and_then(|v| v.as_array()) {
                for item in all_of {
                    if item.get("$ref").is_none() {
                        collect_fields(item);
                    }
                }
            }

            collect_fields(variant);

            if !wrapper_props.is_empty() {
                let target_props = merged_obj
                    .entry("properties".to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                if let Some(target_props_obj) = target_props.as_object_mut() {
                    for (key, value) in wrapper_props {
                        target_props_obj.entry(key).or_insert(value);
                    }
                }
            }

            if !wrapper_required.is_empty() {
                let target_required = merged_obj
                    .entry("required".to_string())
                    .or_insert_with(|| Value::Array(Vec::new()));
                if let Some(target_required_arr) = target_required.as_array_mut() {
                    for req in wrapper_required {
                        if !target_required_arr.contains(&req) {
                            target_required_arr.push(req);
                        }
                    }
                }
            }

            Some(merged)
        }

        fn anchor_text(title: &str) -> String {
            title.to_lowercase()
        }
    }

    #[derive(Default)]
    struct SideDocs {
        agent: HashMap<String, String>,
        client: HashMap<String, String>,
        protocol: HashMap<String, String>,
    }

    impl SideDocs {
        fn agent_method_doc(&self, method_name: &str) -> &String {
            match method_name {
                "initialize" => self.agent.get("InitializeRequest").unwrap(),
                "authenticate" => self.agent.get("AuthenticateRequest").unwrap(),
                "auth/login" => self.agent.get("LoginAuthRequest").unwrap(),
                "providers/list" => self.agent.get("ListProvidersRequest").unwrap(),
                "providers/set" => self.agent.get("SetProviderRequest").unwrap(),
                "providers/disable" => self.agent.get("DisableProviderRequest").unwrap(),
                "session/new" => self.agent.get("NewSessionRequest").unwrap(),
                "session/load" => self.agent.get("LoadSessionRequest").unwrap(),
                "session/list" => self.agent.get("ListSessionsRequest").unwrap(),
                "session/delete" => self.agent.get("DeleteSessionRequest").unwrap(),
                "session/fork" => self.agent.get("ForkSessionRequest").unwrap(),
                "session/resume" => self.agent.get("ResumeSessionRequest").unwrap(),
                "session/set_mode" => self.agent.get("SetSessionModeRequest").unwrap(),
                "session/set_config_option" => {
                    self.agent.get("SetSessionConfigOptionRequest").unwrap()
                }
                "session/prompt" => self.agent.get("PromptRequest").unwrap(),
                "session/inject" => self.agent.get("InjectSessionRequest").unwrap(),
                "session/revoke_inject" => self.agent.get("RevokeInjectSessionRequest").unwrap(),
                "session/replace_inject" => self.agent.get("ReplaceInjectSessionRequest").unwrap(),
                "session/cancel" => self
                    .agent
                    .get("CancelSessionNotification")
                    .or_else(|| self.agent.get("CancelNotification"))
                    .unwrap(),
                "session/close" => self.agent.get("CloseSessionRequest").unwrap(),
                "logout" => self.agent.get("LogoutRequest").unwrap(),
                "auth/logout" => self.agent.get("LogoutAuthRequest").unwrap(),
                "nes/start" => self.agent.get("StartNesRequest").unwrap(),
                "nes/suggest" => self.agent.get("SuggestNesRequest").unwrap(),
                "nes/close" => self.agent.get("CloseNesRequest").unwrap(),
                "nes/accept" => self.agent.get("AcceptNesNotification").unwrap(),
                "nes/reject" => self.agent.get("RejectNesNotification").unwrap(),
                "document/didOpen" => self.agent.get("DidOpenDocumentNotification").unwrap(),
                "document/didChange" => self.agent.get("DidChangeDocumentNotification").unwrap(),
                "document/didClose" => self.agent.get("DidCloseDocumentNotification").unwrap(),
                "document/didSave" => self.agent.get("DidSaveDocumentNotification").unwrap(),
                "document/didFocus" => self.agent.get("DidFocusDocumentNotification").unwrap(),
                "mcp/message" => self.agent.get("MessageMcpRequest").unwrap(),
                _ => panic!("Introduced a method? Add it here :)"),
            }
        }

        fn client_method_doc(&self, method_name: &str) -> &String {
            match method_name {
                "session/request_permission" => {
                    self.client.get("RequestPermissionRequest").unwrap()
                }
                "fs/write_text_file" => self.client.get("WriteTextFileRequest").unwrap(),
                "fs/read_text_file" => self.client.get("ReadTextFileRequest").unwrap(),
                "session/update" => self
                    .client
                    .get("UpdateSessionNotification")
                    .or_else(|| self.client.get("SessionNotification"))
                    .unwrap(),
                "terminal/create" => self.client.get("CreateTerminalRequest").unwrap(),
                "terminal/output" => self.client.get("TerminalOutputRequest").unwrap(),
                "terminal/release" => self.client.get("ReleaseTerminalRequest").unwrap(),
                "terminal/wait_for_exit" => self.client.get("WaitForTerminalExitRequest").unwrap(),
                "terminal/kill" => self.client.get("KillTerminalRequest").unwrap(),
                "elicitation/create" => self.client.get("CreateElicitationRequest").unwrap(),
                "elicitation/complete" => {
                    self.client.get("CompleteElicitationNotification").unwrap()
                }
                "mcp/connect" => self.client.get("ConnectMcpRequest").unwrap(),
                "mcp/message" => self.client.get("MessageMcpRequest").unwrap(),
                "mcp/disconnect" => self.client.get("DisconnectMcpRequest").unwrap(),
                _ => panic!("Introduced a method? Add it here :)"),
            }
        }

        fn protocol_method_doc(&self, method_name: &str) -> &String {
            match method_name {
                "$/cancel_request" => self.protocol.get("CancelRequestNotification").unwrap(),
                _ => panic!("Introduced a method? Add it here :)"),
            }
        }
    }

    #[expect(clippy::too_many_lines)]
    fn extract_side_docs() -> SideDocs {
        let root = super::repo_root();
        let output = Command::new("cargo")
            .current_dir(&root)
            .args([
                "+nightly",
                "rustdoc",
                "-p",
                "agent-client-protocol-schema",
                "--lib",
                "--all-features",
                "--",
                "-Z",
                "unstable-options",
                "--output-format",
                "json",
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "Failed to generate rustdoc JSON: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Parse the JSON output
        let json_path = root.join("target/doc/agent_client_protocol_schema.json");
        let json_content = fs::read_to_string(json_path).unwrap();
        let doc: Value = serde_json::from_str(&json_content).unwrap();

        let mut side_docs = SideDocs::default();

        if let Some(index) = doc["index"].as_object() {
            for (_, item) in index {
                if item["name"].as_str() == Some("ClientRequest")
                    && is_current_protocol_item(item)
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.agent.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }

                if item["name"].as_str() == Some("ClientNotification")
                    && is_current_protocol_item(item)
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.agent.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }

                if item["name"].as_str() == Some("AgentRequest")
                    && is_current_protocol_item(item)
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.client.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }

                if item["name"].as_str() == Some("AgentNotification")
                    && is_current_protocol_item(item)
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.client.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }

                if item["name"].as_str() == Some("ProtocolLevelNotification")
                    && is_current_protocol_item(item)
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.protocol.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }
            }
        }

        side_docs
    }

    fn is_current_protocol_item(item: &Value) -> bool {
        let Some(filename) = item["span"]["filename"].as_str() else {
            return false;
        };

        if cfg!(feature = "unstable_protocol_v2") {
            filename.starts_with("src/v2/")
                || filename.starts_with("agent-client-protocol-schema/src/v2/")
        } else {
            filename.starts_with("src/v1/")
                || filename.starts_with("agent-client-protocol-schema/src/v1/")
        }
    }

    #[cfg(test)]
    mod tests {
        use super::MarkdownGenerator;
        use serde_json::json;

        #[test]
        fn document_union_includes_shared_properties() {
            let mut generator = MarkdownGenerator::new("schema.json");
            let definition = json!({
                "description": "Example union.",
                "discriminator": {
                    "propertyName": "mode"
                },
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Shared message."
                    },
                    "mode": {
                        "type": "string",
                        "description": "The discriminator."
                    }
                },
                "required": ["message", "mode"],
                "oneOf": [
                    {
                        "description": "First variant.",
                        "properties": {
                            "mode": {
                                "const": "form",
                                "type": "string"
                            }
                        },
                        "required": ["mode"],
                        "type": "object"
                    },
                    {
                        "description": "Second variant.",
                        "properties": {
                            "mode": {
                                "const": "url",
                                "type": "string"
                            }
                        },
                        "required": ["mode"],
                        "type": "object"
                    }
                ]
            });

            generator.document_type(4, "ExampleUnion", &definition);

            assert!(generator.output.contains("**Shared properties:**"));
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"message\" type={\"string\"} required>")
            );
            assert!(generator.output.contains("Shared message."));
            assert!(generator.output.contains("**Variants:**"));
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"form\" type=\"object\">")
            );
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"url\" type=\"object\">"),
            );
            let shared_section = generator.output.split("**Variants:**").next().unwrap_or("");
            assert!(
                !shared_section.contains("<ResponseField name=\"mode\""),
                "discriminator property 'mode' should not appear in shared properties"
            );
        }

        #[test]
        fn document_union_renders_both_any_of_and_one_of() {
            let mut generator = MarkdownGenerator::new("schema.json");
            let definition = json!({
                "description": "Request with scope and mode.",
                "anyOf": [
                    {
                        "description": "Session scope.",
                        "properties": {
                            "sessionId": { "type": "string" }
                        },
                        "required": ["sessionId"],
                        "title": "Session",
                        "type": "object"
                    },
                    {
                        "description": "Request scope.",
                        "properties": {
                            "requestId": { "type": "integer" }
                        },
                        "required": ["requestId"],
                        "title": "Request",
                        "type": "object"
                    }
                ],
                "discriminator": { "propertyName": "mode" },
                "oneOf": [
                    {
                        "description": "Form variant.",
                        "properties": {
                            "mode": { "const": "form", "type": "string" }
                        },
                        "required": ["mode"],
                        "type": "object"
                    },
                    {
                        "description": "URL variant.",
                        "properties": {
                            "mode": { "const": "url", "type": "string" }
                        },
                        "required": ["mode"],
                        "type": "object"
                    }
                ],
                "properties": {
                    "message": { "type": "string", "description": "A message." }
                },
                "required": ["message"],
                "type": "object"
            });

            generator.document_type(4, "TestRequest", &definition);

            // Shared properties rendered
            assert!(generator.output.contains("**Shared properties:**"));
            assert!(generator.output.contains("<ResponseField name=\"message\""));

            // anyOf scope variants use title, not "Object"
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"Session\" type=\"object\">"),
                "should use title 'Session' not 'Object'"
            );
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"Request\" type=\"object\">"),
                "should use title 'Request' not 'Object'"
            );

            // oneOf mode variants rendered under Variants
            assert!(generator.output.contains("**Variants:**"));
            assert!(generator.output.contains("<ResponseField name=\"form\""));
            assert!(generator.output.contains("<ResponseField name=\"url\""));

            // Verify ordering: Variants → Session/Request → form/url
            let variants_pos = generator.output.find("**Variants:**").unwrap();
            let session_pos = generator.output.find("\"Session\"").unwrap();
            let form_pos = generator.output.find("\"form\"").unwrap();
            assert!(variants_pos < session_pos);
            assert!(session_pos < form_pos);
        }

        #[test]
        fn document_union_renders_enum_variant_values() {
            let mut generator = MarkdownGenerator::new("schema.json");
            let definition = json!({
                "description": "The sender or recipient.",
                "anyOf": [
                    {
                        "enum": ["assistant", "user"],
                        "type": "string"
                    },
                    {
                        "description": "Custom or future role.",
                        "title": "other",
                        "type": "string"
                    }
                ]
            });

            generator.document_type(4, "Role", &definition);

            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"assistant | user\" type=\"enum\">")
            );
            assert!(
                generator
                    .output
                    .contains("**Values:** `\"assistant\"`, `\"user\"`")
            );
            assert!(!generator.output.contains("<ResponseField name=\"string\""));
        }

        #[test]
        fn document_field_renders_string_and_content_annotations() {
            let mut generator = MarkdownGenerator::new("schema.json");
            let definition = json!({
                "properties": {
                    "count": {
                        "format": "future-number-format",
                        "type": "number"
                    },
                    "payload": {
                        "contentEncoding": "base64",
                        "format": "uri",
                        "type": ["string", "null"]
                    }
                },
                "type": "object"
            });

            generator.document_type(4, "AnnotatedPayload", &definition);

            assert!(generator.output.contains("| Format | `uri` |"));
            assert!(generator.output.contains("| Content encoding | `base64` |"));
            assert!(!generator.output.contains("future-number-format"));
        }

        #[test]
        fn document_simple_type_renders_content_annotations() {
            let mut generator = MarkdownGenerator::new("schema.json");
            let definition = json!({
                "contentEncoding": "base64",
                "contentMediaType": "application/octet-stream",
                "type": "string"
            });

            generator.document_type(4, "EncodedBytes", &definition);

            assert!(generator.output.contains("| Content encoding | `base64` |"));
            assert!(
                generator
                    .output
                    .contains("| Content media type | `application/octet-stream` |")
            );
        }
    }
}
