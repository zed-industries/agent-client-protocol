use agent_client_protocol::{
    AGENT_METHOD_NAMES, AgentNotification, AgentRequest, AgentResponse, CLIENT_METHOD_NAMES,
    ClientNotification, ClientRequest, ClientResponse, VERSION,
};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde_json::Value;
use std::{fs, path::Path};

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(untagged)]
enum AcpTypes {
    ClientRequest(ClientRequest),
    ClientResponse(ClientResponse),
    ClientNotification(ClientNotification),
    AgentRequest(AgentRequest),
    AgentResponse(AgentResponse),
    AgentNotification(AgentNotification),
}

fn main() {
    let mut settings = SchemaSettings::default().for_serialize();
    settings.untagged_enum_variant_titles = true;

    let generator = settings.into_generator();
    let mut schema = generator.into_root_schema_for::<AcpTypes>();
    {
        let schema = schema.as_object_mut().unwrap();
        schema.remove("title");
    }

    // Convert to serde_json::Value for post-processing
    let mut schema_value = serde_json::to_value(&schema).unwrap();

    inline_enum_variants(&mut schema_value, "ContentBlock");
    inline_enum_variants(&mut schema_value, "SessionUpdate");

    let root = env!("CARGO_MANIFEST_DIR");
    let schema_dir = Path::new(root).join("schema");

    fs::create_dir_all(schema_dir.clone()).unwrap();

    fs::write(
        schema_dir.join("schema.json"),
        serde_json::to_string_pretty(&schema_value).unwrap(),
    )
    .expect("Failed to write schema.json");

    // Create a combined metadata object
    let metadata = serde_json::json!({
        "version": VERSION,
        "agentMethods": AGENT_METHOD_NAMES,
        "clientMethods": CLIENT_METHOD_NAMES,
    });

    fs::write(
        schema_dir.join("meta.json"),
        serde_json::to_string_pretty(&metadata).unwrap(),
    )
    .expect("Failed to write meta.json");
}

fn inline_enum_variants(schema: &mut Value, enum_name: &str) {
    let defs_clone = schema.get("$defs").and_then(|v| v.as_object()).cloned();

    let Some(defs_map) = defs_clone else {
        return;
    };

    let Some(defs) = schema.get_mut("$defs").and_then(|v| v.as_object_mut()) else {
        return;
    };

    let Some(enum_def) = defs.get_mut(enum_name).and_then(|v| v.as_object_mut()) else {
        return;
    };

    // Handle both oneOf and anyOf patterns
    let variants = if let Some(one_of) = enum_def.get_mut("oneOf") {
        one_of.as_array_mut()
    } else if let Some(any_of) = enum_def.get_mut("anyOf") {
        any_of.as_array_mut()
    } else {
        None
    };

    let Some(variants) = variants else {
        return;
    };

    for variant in variants.iter_mut() {
        let _ = inline_variant(variant, &defs_map);
    }
}

fn inline_variant(variant: &mut Value, defs_map: &serde_json::Map<String, Value>) -> Option<()> {
    let variant_obj = variant.as_object_mut()?;

    let ref_value = variant_obj.get("$ref")?.as_str()?.to_string();

    let type_name = ref_value.strip_prefix("#/$defs/")?;
    let referenced_type = defs_map.get(type_name)?;

    // Merge properties
    if let Some(ref_props) = referenced_type
        .get("properties")
        .and_then(|v| v.as_object())
    {
        let existing_props = variant_obj
            .entry("properties")
            .or_insert_with(|| Value::Object(serde_json::Map::new()));

        if let Some(existing_obj) = existing_props.as_object_mut() {
            for (key, value) in ref_props {
                existing_obj.insert(key.clone(), value.clone());
            }
        }
    }

    // Merge required fields
    if let Some(ref_required) = referenced_type.get("required").and_then(|v| v.as_array()) {
        let existing_required = variant_obj
            .entry("required")
            .or_insert_with(|| Value::Array(Vec::new()));

        if let Some(existing_arr) = existing_required.as_array_mut() {
            for req in ref_required {
                if !existing_arr.contains(req) {
                    existing_arr.push(req.clone());
                }
            }
        }
    }

    // Remove the $ref
    variant_obj.remove("$ref");

    Some(())
}
