//! Error handling for the Agent Client Protocol.
//!
//! This module provides error types and codes following the JSON-RPC 2.0 specification,
//! with additional protocol-specific error codes for authentication and other ACP-specific scenarios.
//!
//! All methods in the protocol follow standard JSON-RPC 2.0 error handling:
//! - Successful responses include a `result` field
//! - Errors include an `error` object with `code` and `message`
//! - Notifications never receive responses (success or error)
//!
//! See: [Error Handling](https://agentclientprotocol.com/protocol/overview#error-handling)

use std::fmt::Display;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// JSON-RPC error object.
///
/// Represents an error that occurred during method execution, following the
/// JSON-RPC 2.0 error object specification with optional additional data.
///
/// See protocol docs: [JSON-RPC Error Object](https://www.jsonrpc.org/specification#error_object)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Error {
    /// A number indicating the error type that occurred.
    /// This must be an integer as defined in the JSON-RPC specification.
    pub code: i32,
    /// A string providing a short description of the error.
    /// The message should be limited to a concise single sentence.
    pub message: String,
    /// Optional primitive or structured value that contains additional information about the error.
    /// This may include debugging information or context-specific details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Error {
    /// Creates a new error with the given code and message.
    ///
    /// The code parameter can be an `ErrorCode` constant or a tuple of (code, message).
    pub fn new(code: impl Into<(i32, String)>) -> Self {
        let (code, message) = code.into();
        Error {
            code,
            message,
            data: None,
        }
    }

    /// Adds additional data to the error.
    ///
    /// This method is chainable and allows attaching context-specific information
    /// to help with debugging or provide more details about the error.
    #[must_use]
    pub fn with_data(mut self, data: impl Into<serde_json::Value>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
    #[must_use]
    pub fn parse_error() -> Self {
        Error::new(ErrorCode::PARSE_ERROR)
    }

    /// The JSON sent is not a valid Request object.
    #[must_use]
    pub fn invalid_request() -> Self {
        Error::new(ErrorCode::INVALID_REQUEST)
    }

    /// The method does not exist / is not available.
    #[must_use]
    pub fn method_not_found() -> Self {
        Error::new(ErrorCode::METHOD_NOT_FOUND)
    }

    /// Invalid method parameter(s).
    #[must_use]
    pub fn invalid_params() -> Self {
        Error::new(ErrorCode::INVALID_PARAMS)
    }

    /// Internal JSON-RPC error.
    #[must_use]
    pub fn internal_error() -> Self {
        Error::new(ErrorCode::INTERNAL_ERROR)
    }

    /// Authentication required.
    #[must_use]
    pub fn auth_required() -> Self {
        Error::new(ErrorCode::AUTH_REQUIRED)
    }

    /// Converts a standard error into an internal JSON-RPC error.
    ///
    /// The error's string representation is included as additional data.
    pub fn into_internal_error(err: impl std::error::Error) -> Self {
        Error::internal_error().with_data(err.to_string())
    }
}

/// Predefined error codes for common JSON-RPC and ACP-specific errors.
///
/// These codes follow the JSON-RPC 2.0 specification for standard errors
/// and use the reserved range (-32000 to -32099) for protocol-specific errors.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorCode {
    /// The numeric error code.
    pub code: i32,
    /// The standard error message for this code.
    pub message: &'static str,
}

impl ErrorCode {
    /// Invalid JSON was received by the server.
    /// An error occurred on the server while parsing the JSON text.
    pub const PARSE_ERROR: ErrorCode = ErrorCode {
        code: -32700,
        message: "Parse error",
    };

    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: ErrorCode = ErrorCode {
        code: -32600,
        message: "Invalid Request",
    };

    /// The method does not exist or is not available.
    pub const METHOD_NOT_FOUND: ErrorCode = ErrorCode {
        code: -32601,
        message: "Method not found",
    };

    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: ErrorCode = ErrorCode {
        code: -32602,
        message: "Invalid params",
    };

    /// Internal JSON-RPC error.
    /// Reserved for implementation-defined server errors.
    pub const INTERNAL_ERROR: ErrorCode = ErrorCode {
        code: -32603,
        message: "Internal error",
    };

    /// Authentication is required before this operation can be performed.
    /// This is an ACP-specific error code in the reserved range.
    pub const AUTH_REQUIRED: ErrorCode = ErrorCode {
        code: -32000,
        message: "Authentication required",
    };
}

impl From<ErrorCode> for (i32, String) {
    fn from(error_code: ErrorCode) -> Self {
        (error_code.code, error_code.message.to_string())
    }
}

impl From<ErrorCode> for Error {
    fn from(error_code: ErrorCode) -> Self {
        Error::new(error_code)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.message.is_empty() {
            write!(f, "{}", self.code)?;
        } else {
            write!(f, "{}", self.message)?;
        }

        if let Some(data) = &self.data {
            let pretty = serde_json::to_string_pretty(data).unwrap_or_else(|_| data.to_string());
            write!(f, ": {pretty}")?;
        }

        Ok(())
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::into_internal_error(&*error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::invalid_params().with_data(error.to_string())
    }
}
