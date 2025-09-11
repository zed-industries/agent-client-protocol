//! Extension types and constants for protocol extensibility.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Method name for extension methods.
pub const EXT_METHOD_NAME: &str = "_method";
/// Method name for extension notifications.
pub const EXT_NOTIFICATION_NAME: &str = "_notification";

/// Request parameters for extension method calls.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-method" = "_method"))]
#[serde(rename_all = "camelCase")]
pub struct ExtMethodRequest {
    /// The identifier for the extension method.
    ///
    /// To help avoid conflicts, it's a good practice to prefix extension
    /// methods with a unique identifier such as domain name.
    pub method: Arc<str>,
    /// The parameters for the extension method, can be any JSON value.
    pub params: serde_json::Value,
}

/// Response from extension method calls.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-method" = "_method"))]
#[serde(transparent)]
pub struct ExtMethodResponse(pub serde_json::Value);

/// Extension notification parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-method" = "_notification"))]
#[serde(rename_all = "camelCase")]
pub struct ExtNotification {
    /// The identifier for the extension method.
    ///
    /// To help avoid conflicts, it's a good practice to prefix extension
    /// methods with a unique identifier such as domain name.
    pub method: Arc<str>,
    /// The parameters for the extension notification, can be any JSON value.
    pub params: serde_json::Value,
}
