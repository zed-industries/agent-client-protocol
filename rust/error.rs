use std::{fmt::Display, ops::Deref as _};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Error {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Error {
    pub fn new(code: impl Into<(i32, String)>) -> Self {
        let (code, message) = code.into();
        Error {
            code,
            message,
            data: None,
        }
    }

    pub fn with_data(mut self, data: impl Into<serde_json::Value>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
    pub fn parse_error() -> Self {
        Error::new(ErrorCode::PARSE_ERROR)
    }

    /// The JSON sent is not a valid Request object.
    pub fn invalid_request() -> Self {
        Error::new(ErrorCode::INVALID_REQUEST)
    }

    /// The method does not exist / is not available.
    pub fn method_not_found() -> Self {
        Error::new(ErrorCode::METHOD_NOT_FOUND)
    }

    /// Invalid method parameter(s).
    pub fn invalid_params() -> Self {
        Error::new(ErrorCode::INVALID_PARAMS)
    }

    /// Internal JSON-RPC error.
    pub fn internal_error() -> Self {
        Error::new(ErrorCode::INTERNAL_ERROR)
    }

    /// Authentication required.
    pub fn auth_required() -> Self {
        Error::new(ErrorCode::AUTH_REQUIRED)
    }

    pub fn into_internal_error(err: impl std::error::Error) -> Self {
        Error::internal_error().with_data(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorCode {
    pub code: i32,
    pub message: &'static str,
}

impl ErrorCode {
    pub const PARSE_ERROR: ErrorCode = ErrorCode {
        code: -32700,
        message: "Parse error",
    };

    pub const INVALID_REQUEST: ErrorCode = ErrorCode {
        code: -32600,
        message: "Invalid Request",
    };

    pub const METHOD_NOT_FOUND: ErrorCode = ErrorCode {
        code: -32601,
        message: "Method not found",
    };

    pub const INVALID_PARAMS: ErrorCode = ErrorCode {
        code: -32602,
        message: "Invalid params",
    };

    pub const INTERNAL_ERROR: ErrorCode = ErrorCode {
        code: -32603,
        message: "Internal error",
    };

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
            write!(f, ": {data}")?;
        }

        Ok(())
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::into_internal_error(error.deref())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::invalid_params().with_data(error.to_string())
    }
}
