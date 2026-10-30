//! Unified error type for tairu operations.
//!
//! Covers parsing, validation, and serialization of specification data.

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
    /// A JSON document failed to parse.
    #[error("failed to parse {uri}: {source}")]
    Json {
        /// The URI of the document that failed to parse.
        uri: String,
        /// The underlying JSON failure.
        #[source]
        source: serde_json::Error,
    },
    /// A model could not be serialized.
    #[error("failed to serialize {kind}: {source}")]
    Serialize {
        /// The kind of model being serialized.
        kind: &'static str,
        /// The underlying JSON serialization failure.
        #[source]
        source: serde_json::Error,
    },

    /// A subtree envelope ended before its declared content.
    #[error("subtree data truncated")]
    SubtreeTruncated,
    /// A subtree envelope declares an unsupported version.
    #[error("subtree: unsupported version {0}")]
    SubtreeUnsupportedVersion(u32),
    /// The subtree JSON section could not be parsed.
    #[error("subtree JSON: {0}")]
    SubtreeJson(#[source] serde_json::Error),
    /// A subtree buffer view index is out of range.
    #[error("subtree: bufferView index {index} out of range")]
    SubtreeInvalidBufferView {
        /// The offending bufferView index.
        index: usize,
    },
    /// A subtree buffer view range exceeds its binary payload.
    #[error("subtree: buffer view range exceeds binary payload")]
    SubtreeBufferOutOfRange,
    /// A referenced external buffer was not supplied.
    #[error("subtree: external buffer URI '{uri}' not provided")]
    SubtreeMissingExternalBuffer {
        /// The external buffer URI that was not supplied.
        uri: String,
    },
    /// A buffer index in a subtree buffer view is out of range.
    #[error("subtree: buffer index {index} out of range")]
    SubtreeInvalidBufferIndex {
        /// The offending buffer index.
        index: usize,
    },
    /// A subtree section length overflows the host address space.
    #[error("subtree: declared length is too large")]
    SubtreeInvalidLength,

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

    /// Creates an error for a JSON document while retaining its source.
    pub fn json(uri: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Json {
            uri: uri.into(),
            source,
        }
    }

    /// Create an unsupported implicit-subdivision error.
    pub fn unsupported_implicit_subdivision(scheme: &'static str) -> Self {
        Error::UnsupportedImplicitSubdivision { scheme }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_errors_retain_serde_sources() {
        let source = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let error = Error::json("tileset.json", source);
        assert!(
            std::error::Error::source(&error)
                .is_some_and(|source| source.downcast_ref::<serde_json::Error>().is_some())
        );

        let source = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let error = Error::SubtreeJson(source);
        assert!(
            std::error::Error::source(&error)
                .is_some_and(|source| source.downcast_ref::<serde_json::Error>().is_some())
        );
    }
}
