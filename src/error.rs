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
    #[error("failed to fetch {uri}: {source}")]
    Fetch {
        /// The URI that could not be fetched.
        uri: String,
        /// The underlying transport error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
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
}

impl Error {
    /// Create a fetch error from an underlying transport error.
    pub fn fetch(
        uri: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Error::Fetch {
            uri: uri.into(),
            source: Box::new(source),
        }
    }

    /// Create a tile parse error.
    pub fn tile_parse(uri: impl Into<String>, source: TileParseError) -> Self {
        Error::TileParse {
            uri: uri.into(),
            source,
        }
    }

    /// Create a visitor error from the error the visitor returned.
    pub fn visitor(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Error::VisitorFailed {
            source: Box::new(source),
        }
    }

    /// Create an external cycle error.
    pub fn external_cycle(uri: impl Into<String>) -> Self {
        Error::ExternalCycle { uri: uri.into() }
    }
}
