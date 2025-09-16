//! Extension types and constants for protocol extensibility.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(with = "serde_json::Value")]
pub struct ExtRequest {
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    pub params: Arc<RawValue>,
}

pub type ExtResponse = Arc<RawValue>;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(with = "serde_json::Value")]
pub struct ExtNotification {
    #[serde(skip)] // this is used for routing, but when serializing we only want the params
    pub method: Arc<str>,
    pub params: Arc<RawValue>,
}
