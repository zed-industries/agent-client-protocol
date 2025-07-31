use std::{fmt, path::PathBuf, sync::Arc};

use anyhow::Result;
use futures::future::LocalBoxFuture;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Error, SessionId, ToolCall};

pub trait Client {
    fn request_permission(
        &self,
        arguments: RequestPermissionArguments,
    ) -> LocalBoxFuture<'static, Result<RequestPermissionOutput, Error>>;

    fn write_text_file(
        &self,
        arguments: WriteTextFileArguments,
    ) -> LocalBoxFuture<'static, Result<(), Error>>;

    fn read_text_file(
        &self,
        arguments: ReadTextFileArguments,
    ) -> LocalBoxFuture<'static, Result<ReadTextFileOutput, Error>>;
}

pub const REQUEST_PERMISSION_METHOD_NAME: &'static str = "request_permission";
pub const WRITE_TEXT_FILE_METHOD_NAME: &'static str = "write_text_file";
pub const READ_TEXT_FILE_METHOD_NAME: &'static str = "read_text_file";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ClientRequest {
    WriteTextFile(WriteTextFileArguments),
    ReadTextFile(ReadTextFileArguments),
    RequestPermission(RequestPermissionArguments),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ClientResponse {
    WriteTextFile,
    ReadTextFile(ReadTextFileOutput),
    RequestPermission(RequestPermissionOutput),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum ClientNotification {
    Cancelled { request_id: u64 },
}

// Permission

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestPermissionArguments {
    pub session_id: SessionId,
    pub tool_call: ToolCall,
    pub options: Vec<PermissionOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PermissionOption {
    #[serde(rename = "optionId")]
    pub id: PermissionOptionId,
    pub label: String,
    pub kind: PermissionOptionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PermissionOptionId(pub Arc<str>);

impl fmt::Display for PermissionOptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PermissionOptionKind {
    AllowOnce,
    AllowAlways,
    RejectOnce,
    RejectAlways,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestPermissionOutput {
    // This extra-level is unfortunately needed because the output must be an object
    pub outcome: RequestPermissionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum RequestPermissionOutcome {
    Canceled,
    #[serde(rename_all = "camelCase")]
    Selected {
        option_id: PermissionOptionId,
    },
}

// Write text file

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WriteTextFileArguments {
    pub session_id: SessionId,
    pub path: PathBuf,
    pub content: String,
}

// Read text file

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileArguments {
    pub session_id: SessionId,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileOutput {
    pub content: String,
}
