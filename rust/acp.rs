mod agent;
mod client;
mod content;
mod error;
mod plan;
pub mod rpc;
mod tool_call;

pub use agent::*;
pub use client::*;
pub use content::*;
pub use error::*;
pub use plan::*;
pub use tool_call::*;

use std::{fmt, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SessionId(pub Arc<str>);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
