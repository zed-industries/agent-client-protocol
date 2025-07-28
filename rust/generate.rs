use agent_client_protocol::{
    AGENT_METHODS, LoadSessionArguments, NewSessionArguments, NewSessionOutput, PromptArguments,
    ReadTextFileArguments, ReadTextFileOutput, RequestPermissionArguments, RequestPermissionOutput,
    SessionUpdate, WriteTextFileArguments,
};
use schemars::{JsonSchema, generate::SchemaSettings};
use std::fs;

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(untagged)]
enum AcpTypes {
    NewSessionArguments(NewSessionArguments),
    NewSessionOutput(NewSessionOutput),
    LoadSession(LoadSessionArguments),
    Prompt(PromptArguments),
    SessionUpdate(SessionUpdate),
    RequestPermissionArguments(RequestPermissionArguments),
    RequestPermissionOutput(RequestPermissionOutput),
    WriteTextFile(WriteTextFileArguments),
    ReadTextFileArguments(ReadTextFileArguments),
    ReadTextFileOutput(ReadTextFileOutput),
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

    fs::create_dir_all("./schema").unwrap();

    fs::write(
        "./schema/schema.json",
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap();

    fs::write(
        "./schema/methods.json",
        serde_json::to_string_pretty(&AGENT_METHODS).unwrap(),
    )
    .unwrap();
}
