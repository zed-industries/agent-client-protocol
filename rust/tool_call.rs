use std::{path::PathBuf, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ContentBlock;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    #[serde(rename = "toolCallId")]
    pub id: ToolCallId,
    pub title: String,
    pub kind: ToolKind,
    pub status: ToolCallStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ToolCallContent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<ToolCallLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallUpdate {
    #[serde(rename = "toolCallId")]
    pub id: ToolCallId,
    #[serde(flatten)]
    pub fields: ToolCallUpdateFields,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallUpdateFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<ToolKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<ToolCallStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ToolCallContent>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<ToolCallLocation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ToolCallId(pub Arc<str>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    Read,
    Edit,
    Delete,
    Move,
    Search,
    Execute,
    Think,
    Fetch,
    Other,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    /// The tool call is currently running
    Pending,
    /// The tool call is currently running
    InProgress,
    /// The tool call completed successfully
    Completed,
    /// The tool call failed
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolCallContent {
    Content {
        content: ContentBlock,
    },
    Diff {
        #[serde(flatten)]
        diff: Diff,
    },
}

impl<T: Into<ContentBlock>> From<T> for ToolCallContent {
    fn from(content: T) -> Self {
        ToolCallContent::Content {
            content: content.into(),
        }
    }
}

impl From<Diff> for ToolCallContent {
    fn from(diff: Diff) -> Self {
        ToolCallContent::Diff { diff }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Diff {
    pub path: PathBuf,
    pub old_text: Option<String>,
    pub new_text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ToolCallLocation {
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}
