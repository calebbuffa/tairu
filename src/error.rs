//! Unified error type for tairu operations.
//!
//! Covers parsing, validation, and serialization of specification data.

use std::io;

/// All errors that can occur in tairu operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A JSON or schema document failed to parse or validate.
    #[error("failed to parse {uri}: {message}")]
    Parse {
        /// The URI of the tileset that failed to parse.
        uri: String,
        /// The underlying parse failure.
        message: String,
    },
    /// A subtree binary envelope or availability document is invalid.
    #[error("failed to parse subtree: {message}")]
    Subtree {
        /// The underlying parse failure.
        message: String,
    },
    /// A model could not be serialized.
    #[error("failed to serialize {kind}: {message}")]
    Serialize {
        /// The kind of model being serialized.
        kind: &'static str,
        /// The underlying serialization failure.
        message: String,
    },

    /// I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// The loader encountered an implicit tiling scheme it does not implement.
    #[error("unsupported implicit subdivision scheme: {scheme}")]
    UnsupportedImplicitSubdivision {
        /// The unsupported scheme label.
        scheme: &'static str,
    },
}

impl Error {
    /// Creates an error for invalid source or extension data.
    pub fn parse(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Parse {
            uri: uri.into(),
            message: message.into(),
        }
    }

    /// Create a subtree parse error.
    pub fn subtree(message: impl Into<String>) -> Self {
        Error::Subtree {
            message: message.into(),
        }
    }

    /// Create an unsupported implicit-subdivision error.
    pub fn unsupported_implicit_subdivision(scheme: &'static str) -> Self {
        Error::UnsupportedImplicitSubdivision { scheme }
    }
}
