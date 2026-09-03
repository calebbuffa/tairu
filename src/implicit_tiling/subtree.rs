//! Parser for 3D Tiles 1.1 `.subtree` files.
//!
//! Supports both the **binary** envelope (magic `subt`) used by most servers
//! and bare-**JSON** subtrees served without a header.
//!
//! # Binary format layout
//!
//! ```text
//!  0..4   magic    = b"subt"
//!  4..8   version  = 1  (u32 LE)
//!  8..16  json_byte_length  (u64 LE)
//! 16..24  binary_byte_length (u64 LE)
//! 24..    JSON UTF-8
//!         binary blob (buffers referenced by bufferViews)
//! ```
//!
//! The JSON payload describes three availability bitfields:
//! - `tileAvailability` - which tiles in the subtree exist
//! - `contentAvailability` - which tiles have loadable content
//! - `childSubtreeAvailability` - which leaf positions have child subtrees
//!
//! Each field is either `{ "constant": 0|1 }` (all-off / all-on) or
//! `{ "bitstream": N }` (index into `bufferViews`).

use std::collections::HashMap;

use super::{AvailabilityView, SubtreeAvailability};
use crate::generated::{Availability, Buffer, BufferView, SubdivisionScheme, Subtree};

trait LeBytes: Sized + Copy {
    const SIZE: usize;
    fn from_le(bytes: &[u8]) -> Self;
}
macro_rules! impl_le_bytes {
    ($($t:ty),*) => { $(impl LeBytes for $t {
        const SIZE: usize = std::mem::size_of::<$t>();
        fn from_le(b: &[u8]) -> Self { Self::from_le_bytes(b.try_into().unwrap()) }
    })* };
}
impl_le_bytes!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

struct BufferReader<'a> {
    data: &'a [u8],
    pos: usize,
}
impl<'a> BufferReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    fn seek(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n);
    }
    fn read_le<T: LeBytes>(&mut self) -> Option<T> {
        let end = self.pos.checked_add(T::SIZE)?;
        if end > self.data.len() {
            return None;
        }
        let val = T::from_le(&self.data[self.pos..end]);
        self.pos = end;
        Some(val)
    }
    fn read_le_vec<T: LeBytes>(&mut self, count: usize) -> Option<Vec<T>> {
        (0..count).map(|_| self.read_le::<T>()).collect()
    }
    fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        if end > self.data.len() {
            return None;
        }
        let s = &self.data[self.pos..end];
        self.pos = end;
        Some(s)
    }
}

/// Error produced while parsing a `.subtree` file.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SubtreeParseError {
    /// The data was truncated before the end of the declared content.
    #[error("subtree data truncated")]
    TooShort,
    /// Version field is not 1.
    #[error("subtree: unsupported version {0}")]
    BadVersion(u32),
    /// The JSON section could not be parsed.
    #[error("subtree JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// A `bufferView` index in the JSON is out of range.
    #[error("subtree: bufferView index {index} out of range")]
    InvalidBufferView {
        /// The offending bufferView index.
        index: usize,
    },
    /// A buffer view's byte range exceeds the binary blob.
    #[error("subtree: buffer view range exceeds binary payload")]
    BufferOutOfRange,
    /// A `buffer.uri` was referenced but not supplied in `external_buffers`.
    #[error("subtree: external buffer URI '{uri}' not provided")]
    MissingExternalBuffer {
        /// The external buffer URI that was not supplied.
        uri: String,
    },
    /// A `buffer` index in a `bufferView` is out of range.
    #[error("subtree: buffer index {index} out of range")]
    InvalidBufferIndex {
        /// The offending buffer index.
        index: usize,
    },
    /// A declared section or buffer range overflows the host address space.
    #[error("subtree: declared length is too large")]
    InvalidLength,
}

impl SubtreeAvailability {
    /// Decode a JSON or binary `.subtree` payload.
    ///
    /// This variant assumes all buffer data is inline in the payload.
    pub fn from_bytes(
        data: &[u8],
        scheme: SubdivisionScheme,
        subtree_levels: u32,
    ) -> Result<Self, crate::Error> {
        Self::from_bytes_with_buffers(data, &HashMap::new(), scheme, subtree_levels)
    }

    /// Decode a `.subtree` payload with pre-fetched external buffers.
    ///
    /// `external_buffers` maps each `buffer.uri` in the subtree JSON to its raw
    /// bytes. Missing referenced buffers are reported as [`crate::Error`].
    pub fn from_bytes_with_buffers(
        data: &[u8],
        external_buffers: &HashMap<String, Vec<u8>>,
        scheme: SubdivisionScheme,
        subtree_levels: u32,
    ) -> Result<Self, crate::Error> {
        let (json_bytes, inline_binary) =
            split_envelope(data).map_err(|error| crate::Error::subtree(error.to_string()))?;
        let json: Subtree = serde_json::from_slice(json_bytes)
            .map_err(SubtreeParseError::Json)
            .map_err(|error| crate::Error::subtree(error.to_string()))?;
        build_availability(
            &json,
            inline_binary,
            external_buffers,
            scheme,
            subtree_levels,
        )
        .map_err(|error| crate::Error::subtree(error.to_string()))
    }
}

/// Split `data` into *(json_bytes, binary_blob)*.
///
/// If the first four bytes are `subt` the binary header is consumed;
/// otherwise the whole slice is treated as JSON with an empty binary blob.
fn split_envelope(data: &[u8]) -> Result<(&[u8], &[u8]), SubtreeParseError> {
    const MAGIC: &[u8; 4] = b"subt";
    const HEADER: usize = 24; // 4 magic + 4 version + 8 json_len + 8 bin_len

    if data.len() >= 4 && data[..4] == *MAGIC {
        if data.len() < HEADER {
            return Err(SubtreeParseError::TooShort);
        }
        let mut r = BufferReader::new(data);
        r.seek(4); // skip magic
        let version = r.read_le::<u32>().ok_or(SubtreeParseError::TooShort)?;
        if version != 1 {
            return Err(SubtreeParseError::BadVersion(version));
        }
        // Read both 64-bit length fields together as a batch.
        let lens = r.read_le_vec::<u64>(2).ok_or(SubtreeParseError::TooShort)?;
        let json_len = usize::try_from(lens[0]).map_err(|_| SubtreeParseError::InvalidLength)?;
        let bin_len = usize::try_from(lens[1]).map_err(|_| SubtreeParseError::InvalidLength)?;

        let json_start = HEADER;
        let json_end = json_start
            .checked_add(json_len)
            .ok_or(SubtreeParseError::InvalidLength)?;
        let bin_end = json_end
            .checked_add(bin_len)
            .ok_or(SubtreeParseError::InvalidLength)?;

        if data.len() < bin_end {
            return Err(SubtreeParseError::TooShort);
        }
        Ok((&data[json_start..json_end], &data[json_end..bin_end]))
    } else {
        // Plain JSON - no binary blob.
        Ok((data, &[]))
    }
}

/// Resolve an [`Availability`] to an [`AvailabilityView`].
///
/// Bitstream specs copy bytes from either the inline binary blob (buffer
/// with no `uri`) or a pre-fetched external buffer (buffer with `uri`).
fn resolve_spec(
    spec: &Availability,
    buffer_views: &[BufferView],
    buffers: &[Buffer],
    inline_binary: &[u8],
    external_buffers: &HashMap<String, Vec<u8>>,
) -> Result<AvailabilityView, SubtreeParseError> {
    if let Some(c) = spec.constant {
        return Ok(AvailabilityView::Constant(c != 0));
    }
    if let Some(bv_idx) = spec.bitstream {
        let bv = buffer_views
            .get(bv_idx)
            .ok_or(SubtreeParseError::InvalidBufferView { index: bv_idx })?;

        // Resolve the buffer this view belongs to.
        let buffer_data: &[u8] = if buffers.is_empty() {
            // Legacy subtrees with no explicit buffers array - use inline blob.
            inline_binary
        } else {
            let buf = buffers
                .get(bv.buffer)
                .ok_or(SubtreeParseError::InvalidBufferIndex { index: bv.buffer })?;
            if let Some(uri) = &buf.uri {
                external_buffers
                    .get(uri.as_str())
                    .map(Vec::as_slice)
                    .ok_or_else(|| SubtreeParseError::MissingExternalBuffer { uri: uri.clone() })?
            } else {
                inline_binary
            }
        };

        let end = bv
            .byte_offset
            .checked_add(bv.byte_length)
            .ok_or(SubtreeParseError::BufferOutOfRange)?;
        if end > buffer_data.len() {
            return Err(SubtreeParseError::BufferOutOfRange);
        }
        // Use BufferReader so we go through the same bounds-checked path
        // as the rest of the binary parsing.
        let mut reader = BufferReader::new(buffer_data);
        reader.seek(bv.byte_offset);
        let bytes = reader
            .read_bytes(bv.byte_length)
            .ok_or(SubtreeParseError::BufferOutOfRange)?;
        return Ok(AvailabilityView::Bitstream(bytes.to_vec()));
    }
    // Neither constant nor bitstream - treat as all-unavailable.
    Ok(AvailabilityView::Constant(false))
}

/// Convert the parsed JSON into a [`SubtreeAvailability`].
fn build_availability(
    json: &Subtree,
    inline_binary: &[u8],
    external_buffers: &HashMap<String, Vec<u8>>,
    scheme: SubdivisionScheme,
    subtree_levels: u32,
) -> Result<SubtreeAvailability, SubtreeParseError> {
    let resolve = |spec: &Availability| {
        resolve_spec(
            spec,
            &json.buffer_views,
            &json.buffers,
            inline_binary,
            external_buffers,
        )
    };

    let tile_av = resolve(&json.tile_availability)?;
    let child_subtree_av = resolve(&json.child_subtree_availability)?;

    let content_av: Result<Vec<AvailabilityView>, _> = if json.content_availability.is_empty() {
        // Spec says contentAvailability may be omitted - default to all-unavailable.
        Ok(vec![AvailabilityView::Constant(false)])
    } else {
        json.content_availability.iter().map(resolve).collect()
    };
    let content_av = content_av?;

    // The parser always supplies at least one content layer, so construct the
    // invariant-preserving value directly and avoid an unreachable panic path.
    Ok(SubtreeAvailability::from_parts(
        scheme,
        subtree_levels,
        tile_av,
        child_subtree_av,
        content_av,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_available_json() -> Vec<u8> {
        br#"{
            "tileAvailability": { "constant": 1 },
            "contentAvailability": [{ "constant": 1 }],
            "childSubtreeAvailability": { "constant": 0 }
        }"#
        .to_vec()
    }

    fn wrap_binary(json: &[u8], binary: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"subt");
        out.extend_from_slice(&1u32.to_le_bytes());
        out.extend_from_slice(&(json.len() as u64).to_le_bytes());
        out.extend_from_slice(&(binary.len() as u64).to_le_bytes());
        out.extend_from_slice(json);
        out.extend_from_slice(binary);
        out
    }

    #[test]
    fn plain_json_all_available() {
        let data = all_available_json();
        let sa = SubtreeAvailability::from_bytes(&data, SubdivisionScheme::Quadtree, 2).unwrap();
        assert!(sa.is_tile_available_at(0, 0));
        for m in 0..4 {
            assert!(sa.is_tile_available_at(1, m));
        }
    }

    #[test]
    fn binary_envelope_all_available() {
        let json = all_available_json();
        let data = wrap_binary(&json, &[]);
        let sa = SubtreeAvailability::from_bytes(&data, SubdivisionScheme::Quadtree, 2).unwrap();
        assert!(sa.is_tile_available_at(0, 0));
    }

    #[test]
    fn constant_unavailable() {
        let json = br#"{
            "tileAvailability": { "constant": 0 },
            "contentAvailability": [],
            "childSubtreeAvailability": { "constant": 0 }
        }"#;
        let sa = SubtreeAvailability::from_bytes(json, SubdivisionScheme::Quadtree, 2).unwrap();
        assert!(!sa.is_tile_available_at(0, 0));
    }

    #[test]
    fn bitstream_partial_availability() {
        // Level-2 quadtree subtree: 1 + 4 + 16 = 21 tiles -> 3 bytes.
        // Mark only the root (bit 0) available.
        let bits: Vec<u8> = vec![0b0000_0001, 0x00, 0x00];
        let json = format!(
            r#"{{
                "bufferViews": [{{"buffer":0,"byteOffset":0,"byteLength":{}}}],
                "tileAvailability": {{"bitstream":0}},
                "contentAvailability": [{{"constant":0}}],
                "childSubtreeAvailability": {{"constant":0}}
            }}"#,
            bits.len()
        );
        let data = wrap_binary(json.as_bytes(), &bits);
        let sa = SubtreeAvailability::from_bytes(&data, SubdivisionScheme::Quadtree, 2).unwrap();
        assert!(sa.is_tile_available_at(0, 0), "root must be available");
        assert!(
            !sa.is_tile_available_at(1, 0),
            "level-1 tiles must be unavailable"
        );
    }

    #[test]
    fn truncated_binary_is_error() {
        let json = all_available_json();
        let mut data = wrap_binary(&json, &[]);
        data.truncate(10); // cut short
        assert!(matches!(
            SubtreeAvailability::from_bytes(&data, SubdivisionScheme::Quadtree, 2),
            Err(crate::Error::Subtree { .. })
        ));
    }

    #[test]
    fn bad_version_is_error() {
        let json = all_available_json();
        let mut data = wrap_binary(&json, &[]);
        // Overwrite version field (bytes 4..8) with 2.
        data[4..8].copy_from_slice(&2u32.to_le_bytes());
        assert!(matches!(
            SubtreeAvailability::from_bytes(&data, SubdivisionScheme::Quadtree, 2),
            Err(crate::Error::Subtree { .. })
        ));
    }

    #[test]
    fn overflowing_declared_lengths_are_rejected() {
        let json = all_available_json();
        let mut data = Vec::new();
        data.extend_from_slice(b"subt");
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&u64::MAX.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());
        data.extend_from_slice(&json);
        assert!(matches!(
            SubtreeAvailability::from_bytes(&data, SubdivisionScheme::Quadtree, 2),
            Err(crate::Error::Subtree { .. })
        ));
    }
}
