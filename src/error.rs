//! Unified error type for tairu operations.
//!
//! All tairu public API operations return a single [`Error`] type,
//! though internally we may compose multiple specialized error types.

use crate::reader::TileParseError;
use crate::subtree::SubtreeParseError;
use std::io;

/// All errors that can occur in tairu operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to fetch a resource.
    #[error("failed to fetch {uri}: {reason}")]
    Fetch {
        /// The URI that could not be fetched.
        uri: String,
        /// Human-readable failure detail.
        reason: String,
    },

    /// Failed to parse tileset JSON.
    #[error("failed to parse tileset {uri}: {source}")]
    TileParse {
        /// The URI of the tileset that failed to parse.
        uri: String,
        /// The underlying parse error.
        #[source]
        source: TileParseError,
    },

    /// Failed to parse subtree data.
    #[error("failed to parse subtree: {0}")]
    SubtreeParse(#[from] SubtreeParseError),

    /// I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// User's visitor callback failed.
    #[error("tile visitor failed: {0}")]
    VisitorFailed(String),

    /// External tileset cycle detected during traversal.
    #[error("external tileset cycle detected at {uri}")]
    ExternalCycle {
        /// The URI where the cycle was detected.
        uri: String,
    },

    /// URI parsing or resolution failed.
    #[error("invalid URI {uri}: {reason}")]
    InvalidUri {
        /// The malformed URI.
        uri: String,
        /// Human-readable failure detail.
        reason: String,
    },
}

impl Error {
    /// Create a fetch error with the given URI and reason.
    pub fn fetch(uri: impl Into<String>, reason: impl Into<String>) -> Self {
        Error::Fetch {
            uri: uri.into(),
            reason: reason.into(),
        }
    }

    /// Create a tile parse error.
    pub fn tile_parse(uri: impl Into<String>, source: TileParseError) -> Self {
        Error::TileParse {
            uri: uri.into(),
            source,
        }
    }

    /// Create a visitor error from a Display-able type.
    pub fn visitor(reason: impl std::fmt::Display) -> Self {
        Error::VisitorFailed(reason.to_string())
    }

    /// Create an external cycle error.
    pub fn external_cycle(uri: impl Into<String>) -> Self {
        Error::ExternalCycle { uri: uri.into() }
    }

    /// Create an invalid URI error.
    pub fn invalid_uri(uri: impl Into<String>, reason: impl Into<String>) -> Self {
        Error::InvalidUri {
            uri: uri.into(),
            reason: reason.into(),
        }
    }
}
