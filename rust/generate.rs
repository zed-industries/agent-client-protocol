use agent_client_protocol::{
    LoadSessionToolArguments, NewSessionToolArguments, PermissionOutcome, PermissionToolArguments,
    PromptToolArguments, ReadTextFileArguments, SessionUpdate, WriteTextFileToolArguments,
};
use schemars::{JsonSchema, generate::SchemaSettings};
use std::fs;

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(untagged)]
enum Acp {
    NewSession(NewSessionToolArguments),
    LoadSession(LoadSessionToolArguments),
    Prompt(PromptToolArguments),
    SessionUpdate(SessionUpdate),
    PermissionTool(PermissionToolArguments, PermissionOutcome),
    WriteTextFile(WriteTextFileToolArguments),
    ReadTextFile(ReadTextFileArguments),
}

fn main() {
    let settings = SchemaSettings::default().for_serialize();
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
