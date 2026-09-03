//! Unified error type for tairu operations.
//!
//! All tairu public API operations return a single [`Error`] type,
//! though internally we may compose multiple specialized error types.

use std::io;

/// All errors that can occur in tairu operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to fetch a resource.
    #[error("failed to fetch {uri}: {source}")]
    Fetch {
        /// The URI that could not be fetched.
        uri: String,
        /// The underlying transport error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

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

    /// User's visitor callback failed.
    #[error("tile visitor failed: {source}")]
    VisitorFailed {
        /// The error returned by the visitor.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// External tileset cycle detected during traversal.
    #[error("external tileset cycle detected at {uri}")]
    ExternalCycle {
        /// The URI where the cycle was detected.
        uri: String,
    },

    /// Traversal exceeded its built-in depth limit.
    #[error("tileset traversal exceeded maximum depth {max}")]
    TraversalLimit {
        /// Maximum permitted traversal depth.
        max: usize,
    },
}

impl Error {
    /// Create a fetch error from an underlying transport error.
    pub(crate) fn fetch(
        uri: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Error::Fetch {
            uri: uri.into(),
            source: source.into(),
        }
    }

    /// Create a tile parse error.
    pub(crate) fn parse(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Parse {
            uri: uri.into(),
            message: message.into(),
        }
    }

    /// Create a subtree parse error.
    pub(crate) fn subtree(message: impl Into<String>) -> Self {
        Error::Subtree {
            message: message.into(),
        }
    }

    /// Create a visitor error from the error the visitor returned.
    pub(crate) fn visitor(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Error::VisitorFailed {
            source: Box::new(source),
        }
    }

    /// Create an external cycle error.
    pub(crate) fn external_cycle(uri: impl Into<String>) -> Self {
        Error::ExternalCycle { uri: uri.into() }
    }
}
