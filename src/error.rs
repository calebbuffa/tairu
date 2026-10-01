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

    /// The loader encountered an implicit tiling scheme it does not implement.
    #[error("unsupported implicit subdivision scheme: {scheme}")]
    UnsupportedImplicitSubdivision {
        /// The unsupported scheme label.
        scheme: &'static str,
    },

    /// A CPU job submitted to a caller-provided [`hiera::Spawner`] failed.
    #[error("spawned parse job failed: {0}")]
    Spawn(#[source] hiera::SpawnError),
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
    /// Creates an error for invalid source or extension data.
    pub fn parse(uri: impl Into<String>, message: impl Into<String>) -> Self {
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

    /// Create an unsupported implicit-subdivision error.
    pub(crate) fn unsupported_implicit_subdivision(scheme: &'static str) -> Self {
        Error::UnsupportedImplicitSubdivision { scheme }
    }

    /// Create a spawn error from a failed [`hiera::Spawner`] job.
    pub(crate) fn spawn(source: hiera::SpawnError) -> Self {
        Error::Spawn(source)
    }
}
