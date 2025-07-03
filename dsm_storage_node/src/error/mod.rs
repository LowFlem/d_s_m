// Error handling module for DSM Storage Node
//
// This module defines error types and utility functions for error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::io;
use std::result;
use thiserror::Error;

/// Result type for DSM Storage Node operations
pub type Result<T> = result::Result<T, StorageNodeError>;

/// Error type for DSM Storage Node operations
#[derive(Debug, Error, Clone)]
pub enum StorageNodeError {
    /// Network client not set
    #[error("Network client not set")]
    NetworkClientNotSet,

    /// Invalid node ID
    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),

    /// Operation not permitted or invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Configuration error
    #[error("Configuration error")]
    Configuration,

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Storage-related errors
    #[error("Storage error: {0}")]
    Storage(String),

    /// Cryptographic errors
    #[error("Crypto error: {message}")]
    CryptoError { message: String },

    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Encryption-related errors
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Distribution-related errors
    #[error("Distribution error: {0}")]
    Distribution(String),

    /// Node management-related errors
    #[error("Node management error: {0}")]
    NodeManagement(String),

    /// Staking-related errors
    #[error("Staking error: {0}")]
    Staking(String),

    /// Authentication-related errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Invalid state errors
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Database-related errors
    #[error("Database error: {0}")]
    Database(String),

    /// Serialization-related errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Network-related errors
    #[error("Network error: {0}")]
    Network(String),

    /// IO errors
    #[error("IO error: {0}")]
    IO(String),

    /// JSON errors
    #[error("JSON error: {0}")]
    Json(String),

    /// SQLite errors
    #[error("SQLite error: {0}")]
    Sqlite(String),

    /// HTTP request errors
    #[error("Request error: {0}")]
    Request(String),

    /// Unknown errors
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Concurrency limit exceeded
    #[error("Concurrency limit exceeded")]
    ConcurrencyLimitExceeded,

    /// Task cancelled
    #[error("Task cancelled: {0}")]
    TaskCancelled(String),

    /// Task failed
    #[error("Task failed: {0}")]
    TaskFailed(String),

    /// Queue full
    #[error("Queue full: {0}")]
    QueueFull(String),

    /// Receive failure
    #[error("Receive failure: {0}")]
    ReceiveFailure(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Bad request
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Cryptographic operation errors
    #[error("Cryptographic error: {context}")]
    Crypto { context: String },

    /// Integrity check failures
    #[error("Integrity error: {context}")]
    Integrity { context: String },

    /// Invalid public key error
    #[error("Invalid public key")]
    InvalidPublicKey,

    /// Invalid secret/private key error
    #[error("Invalid secret key")]
    InvalidSecretKey,

    /// Invalid ciphertext error
    #[error("Invalid ciphertext")]
    InvalidCiphertext,

    /// Invalid key length error
    #[error("Invalid key length")]
    InvalidKeyLength,

    /// State machine errors
    #[error("State machine error: {0}")]
    StateMachine(String),

    /// Genesis/Identity related errors
    #[error("Genesis error: {0}")]
    Genesis(String),

    /// Policy related errors
    #[error("Policy error: {0}")]
    Policy(String),

    /// Vault/DLV related errors
    #[error("Vault error: {0}")]
    Vault(String),
}

impl StorageNodeError {
    /// Create a cryptographic error
    pub fn crypto<S: Into<String>>(context: S) -> Self {
        Self::Crypto {
            context: context.into(),
        }
    }

    /// Create an integrity error
    pub fn integrity<S: Into<String>>(context: S) -> Self {
        Self::Integrity {
            context: context.into(),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    /// Create a storage error
    pub fn storage<S: Into<String>>(message: S) -> Self {
        Self::Storage(message.into())
    }

    /// Create a serialization error
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        Self::Serialization(message.into())
    }

    /// Create a genesis error
    pub fn genesis<S: Into<String>>(message: S) -> Self {
        Self::Genesis(message.into())
    }

    /// Create a policy error
    pub fn policy<S: Into<String>>(message: S) -> Self {
        Self::Policy(message.into())
    }

    /// Create a vault error
    pub fn vault<S: Into<String>>(message: S) -> Self {
        Self::Vault(message.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S, _source: Option<std::io::Error>) -> Self {
        Self::Internal(message.into())
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(message: S, _source: Option<std::io::Error>) -> Self {
        Self::NotFound(message.into())
    }
}

/// Implement IntoResponse for StorageNodeError so it can be returned directly from handlers
impl IntoResponse for StorageNodeError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            StorageNodeError::NetworkClientNotSet => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Network client not configured".to_string(),
            ),
            StorageNodeError::InvalidNodeId(msg) => (StatusCode::BAD_REQUEST, msg),
            StorageNodeError::InvalidOperation(msg) => (StatusCode::BAD_REQUEST, msg),
            StorageNodeError::Timeout => (
                StatusCode::REQUEST_TIMEOUT,
                "Operation timed out".to_string(),
            ),
            StorageNodeError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Configuration => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration error".to_string(),
            ),
            StorageNodeError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            StorageNodeError::InvalidState(msg) => (StatusCode::BAD_REQUEST, msg),
            StorageNodeError::Authentication(msg) => (StatusCode::UNAUTHORIZED, msg),
            StorageNodeError::Storage(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::CryptoError { message } => (StatusCode::INTERNAL_SERVER_ERROR, message),
            StorageNodeError::Encryption(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Distribution(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::NodeManagement(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Staking(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Serialization(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::IO(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
            StorageNodeError::Json(err) => (StatusCode::BAD_REQUEST, err),
            StorageNodeError::Sqlite(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
            StorageNodeError::Request(err) => (StatusCode::BAD_GATEWAY, err),
            StorageNodeError::Network(err) => (StatusCode::BAD_GATEWAY, err),
            StorageNodeError::Unknown(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::RateLimitExceeded(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            StorageNodeError::TaskCancelled(msg) => (StatusCode::CONFLICT, msg),
            StorageNodeError::ConcurrencyLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Concurrency limit exceeded".to_string(),
            ),
            StorageNodeError::TaskFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::QueueFull(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            StorageNodeError::ReceiveFailure(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            StorageNodeError::NotImplemented(msg) => (StatusCode::NOT_IMPLEMENTED, msg),
            StorageNodeError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            StorageNodeError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            StorageNodeError::InvalidConfiguration(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Crypto { context } => (StatusCode::INTERNAL_SERVER_ERROR, context),
            StorageNodeError::Integrity { context } => (StatusCode::INTERNAL_SERVER_ERROR, context),
            StorageNodeError::InvalidPublicKey => {
                (StatusCode::BAD_REQUEST, "Invalid public key".to_string())
            }
            StorageNodeError::InvalidSecretKey => {
                (StatusCode::BAD_REQUEST, "Invalid secret key".to_string())
            }
            StorageNodeError::InvalidKeyLength => {
                (StatusCode::BAD_REQUEST, "Invalid key length".to_string())
            }
            StorageNodeError::InvalidCiphertext => {
                (StatusCode::BAD_REQUEST, "Invalid ciphertext".to_string())
            }
            StorageNodeError::StateMachine(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Genesis(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Policy(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            StorageNodeError::Vault(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": error_message
            }
        }));

        (status, body).into_response()
    }
}

// Implement conversion from rusqlite error to StorageNodeError
impl From<rusqlite::Error> for StorageNodeError {
    fn from(err: rusqlite::Error) -> Self {
        StorageNodeError::Sqlite(err.to_string())
    }
}

// Implement conversion from io::Error to StorageNodeError
impl From<io::Error> for StorageNodeError {
    fn from(err: io::Error) -> Self {
        StorageNodeError::IO(err.to_string())
    }
}

// Implement conversion from reqwest error to StorageNodeError
impl From<reqwest::Error> for StorageNodeError {
    fn from(err: reqwest::Error) -> Self {
        StorageNodeError::Request(err.to_string())
    }
}

// Implement conversion from toml serialization error to StorageNodeError
impl From<toml::ser::Error> for StorageNodeError {
    fn from(err: toml::ser::Error) -> Self {
        StorageNodeError::Serialization(err.to_string())
    }
}

// Implement conversion from toml deserialization error to StorageNodeError
impl From<toml::de::Error> for StorageNodeError {
    fn from(err: toml::de::Error) -> Self {
        StorageNodeError::Serialization(err.to_string())
    }
}

// Implement conversion from serde_json::Error to StorageNodeError
impl From<serde_json::Error> for StorageNodeError {
    fn from(err: serde_json::Error) -> Self {
        StorageNodeError::Json(err.to_string())
    }
}

// Implement conversion from bincode::ErrorKind to StorageNodeError
impl From<Box<bincode::ErrorKind>> for StorageNodeError {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        StorageNodeError::Serialization(err.to_string())
    }
}

// Implement conversion from reqwest::StatusCode to StorageNodeError
impl From<reqwest::StatusCode> for StorageNodeError {
    fn from(status: reqwest::StatusCode) -> Self {
        StorageNodeError::Request(format!("HTTP error status: {status}"))
    }
}
