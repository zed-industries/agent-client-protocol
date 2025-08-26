use schemars::Schema;
use schemars::transform::Transform;
use serde_json::json;

/// Transform that adds grouping metadata to schemas for documentation generation.
///
/// This transform adds `x-group` and `x-order` extension fields to schemas,
/// which are used by the documentation generator to organize types into
/// logical sections.
pub struct GroupMetadata {
    group: &'static str,
    order: u32,
}

impl GroupMetadata {
    /// Create a transform for initialization-related types
    pub fn initialization() -> Self {
        Self {
            group: "initialization",
            order: 2,
        }
    }

    /// Create a transform for session management types
    pub fn session() -> Self {
        Self {
            group: "session",
            order: 3,
        }
    }

    /// Create a transform for content and resource types
    pub fn content() -> Self {
        Self {
            group: "content",
            order: 4,
        }
    }

    /// Create a transform for tool and permission types
    pub fn tools() -> Self {
        Self {
            group: "tools",
            order: 5,
        }
    }

    /// Create a transform for file system types
    pub fn filesystem() -> Self {
        Self {
            group: "filesystem",
            order: 6,
        }
    }

    /// Create a transform for MCP configuration types
    pub fn mcp() -> Self {
        Self {
            group: "mcp",
            order: 7,
        }
    }

    /// Create a transform for core message types
    pub fn message_types() -> Self {
        Self {
            group: "message-types",
            order: 1,
        }
    }
}

impl Transform for GroupMetadata {
    fn transform(&mut self, schema: &mut Schema) {
        // Work with Schema as a JSON value
        if let Ok(mut value) = serde_json::to_value(&schema) {
            if let Some(obj) = value.as_object_mut() {
                // Add x-group extension for categorization
                obj.insert("x-group".to_string(), json!(self.group));

                // Add x-order extension for sorting within documentation
                obj.insert("x-order".to_string(), json!(self.order));
            }

            // Convert back to Schema
            if let Ok(new_schema) = serde_json::from_value(value) {
                *schema = new_schema;
            }
        }
    }
}

/// Helper function for initialization types
pub fn add_group_initialization(schema: &mut Schema) {
    let mut metadata = GroupMetadata::initialization();
    metadata.transform(schema);
}

/// Helper function for session types
pub fn add_group_session(schema: &mut Schema) {
    let mut metadata = GroupMetadata::session();
    metadata.transform(schema);
}

/// Helper function for content types
pub fn add_group_content(schema: &mut Schema) {
    let mut metadata = GroupMetadata::content();
    metadata.transform(schema);
}

/// Helper function for tool types
pub fn add_group_tools(schema: &mut Schema) {
    let mut metadata = GroupMetadata::tools();
    metadata.transform(schema);
}

/// Helper function for filesystem types
pub fn add_group_filesystem(schema: &mut Schema) {
    let mut metadata = GroupMetadata::filesystem();
    metadata.transform(schema);
}

/// Helper function for MCP types
pub fn add_group_mcp(schema: &mut Schema) {
    let mut metadata = GroupMetadata::mcp();
    metadata.transform(schema);
}

/// Helper function for message types
pub fn add_group_message_types(schema: &mut Schema) {
    let mut metadata = GroupMetadata::message_types();
    metadata.transform(schema);
}
