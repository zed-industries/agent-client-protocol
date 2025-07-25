use agent_client_protocol::{
    LoadSessionArguments, NewSessionArguments, NewSessionOutput, PromptArguments,
    ReadTextFileArguments, ReadTextFileOutput, RequestPermissionArguments, RequestPermissionOutput,
    SessionUpdate, WriteTextFileArguments,
};
use schemars::{JsonSchema, generate::SchemaSettings};
use std::fs;

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(untagged)]
enum Acp {
    NewSession(NewSessionArguments, NewSessionOutput),
    LoadSession(LoadSessionArguments),
    Prompt(PromptArguments),
    SessionUpdate(SessionUpdate),
    PermissionTool(RequestPermissionArguments, RequestPermissionOutput),
    WriteTextFile(WriteTextFileArguments),
    ReadTextFile(ReadTextFileArguments, ReadTextFileOutput),
}

fn main() {
    let mut settings = SchemaSettings::default().for_serialize();
    settings.untagged_enum_variant_titles = true;

    let generator = settings.into_generator();
    let mut schema = generator.into_root_schema_for::<Acp>();
    {
        let schema = schema.as_object_mut().unwrap();
        schema.remove("title");
    }

    fs::write(
        "./schema.json",
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap();
}
