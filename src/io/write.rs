//! Writers for 3D Tiles objects - [`TilesetWriter`], [`SubtreeWriter`], [`SchemaWriter`].
//!
//! Writers return ordinary [`crate::Error`] values for serialization failures.
//!
//! # Tileset example
//!
//! ```no_run
//! use tairu::{Tileset, Asset, Tile, BoundingVolume, Refine, TilesetWriter, WriteOptions};
//!
//! let tileset = Tileset {
//!     asset: Asset { version: "1.1".into(), ..Default::default() },
//!     geometric_error: 1000.0,
//!     root: Tile {
//!         bounding_volume: BoundingVolume { sphere: Some([0.0, 0.0, 0.0, 1000.0]), ..Default::default() },
//!         geometric_error: 1000.0,
//!         refine: Some(Refine::Replace),
//!         ..Default::default()
//!     },
//!     ..Default::default()
//! };
//!
//! let bytes = TilesetWriter::write_tileset(&tileset, WriteOptions::default()).unwrap();
//! std::fs::write("tileset.json", &bytes).unwrap();
//! ```
//!
//! # Subtree example
//!
//! ```no_run
//! use tairu::{Subtree, Availability, SubtreeWriter, WriteOptions};
//!
//! let subtree = Subtree {
//!     tile_availability: Availability { constant: Some(1), ..Default::default() },
//!     child_subtree_availability: Availability { constant: Some(0), ..Default::default() },
//!     ..Default::default()
//! };
//!
//! // Write JSON subtree.
//! let result = SubtreeWriter::write_subtree_json(&subtree, WriteOptions::default()).unwrap();
//!
//! // Write binary subtree (first buffer is the inline binary chunk).
//! let binary_payload = vec![0u8; 16];
//! let result = SubtreeWriter::write_subtree_binary(&subtree, &binary_payload, WriteOptions::default()).unwrap();
//! ```

use crate::generated::{Buffer, Schema, Subtree, Tileset};
/// Minimal append-only byte buffer writer.
struct BufferWriter(Vec<u8>);
impl BufferWriter {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn with_capacity(n: usize) -> Self {
        Self(Vec::with_capacity(n))
    }
    fn write_le<T: WriteLeBytes>(&mut self, v: T) {
        v.write_le(&mut self.0);
    }
    fn write_bytes(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b);
    }
    /// Pad to the next multiple of `align` bytes using `fill`.
    fn align_to(&mut self, align: usize, fill: u8) {
        let rem = self.0.len() % align;
        if rem != 0 {
            self.0.extend(std::iter::repeat_n(fill, align - rem));
        }
    }
    fn finish(self) -> Vec<u8> {
        self.0
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}
trait WriteLeBytes {
    fn write_le(self, buf: &mut Vec<u8>);
}
macro_rules! impl_write_le { ($($t:ty),*) => { $(impl WriteLeBytes for $t {
    fn write_le(self, buf: &mut Vec<u8>) { buf.extend_from_slice(&self.to_le_bytes()); }
})* }; }
impl_write_le!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

/// Options controlling serialization behaviour.
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Emit pretty-printed (indented) JSON. Default: compact.
    pub pretty_print: bool,
}

/// Serializes a [`Tileset`] to JSON bytes.
pub struct TilesetWriter;

impl TilesetWriter {
    /// Serialize a tileset to JSON.
    pub fn write_tileset(tileset: &Tileset, opts: WriteOptions) -> Result<Vec<u8>, crate::Error> {
        serialize(tileset, opts.pretty_print, "tileset")
    }
}

/// Serializes a [`Subtree`] to JSON or the binary `.subtree` envelope.
///
/// ## Binary envelope layout
///
/// ```text
///  0.. 4  magic             = b"subt"
///  4.. 8  version           = 1  (u32 LE)
///  8..16  json_byte_length  (u64 LE, padded to 8-byte alignment)
/// 16..24  binary_byte_length (u64 LE)
/// 24..    JSON UTF-8 + alignment padding (0x20)
///         binary blob
/// ```
pub struct SubtreeWriter;

impl SubtreeWriter {
    /// Serialize a subtree to JSON (`.subtree` JSON form).
    ///
    /// External buffer URIs (`buffer.uri`) must be set on the subtree before
    /// calling this; the binary payload lives in separate files.
    pub fn write_subtree_json(
        subtree: &Subtree,
        opts: WriteOptions,
    ) -> Result<Vec<u8>, crate::Error> {
        let bytes = serialize(subtree, opts.pretty_print, "subtree")?;
        let mut w = BufferWriter::new();
        w.write_bytes(&bytes);
        Ok(w.finish())
    }

    /// Serialize a subtree to the binary `.subtree` envelope.
    ///
    /// `buffer_data` is the inline binary payload appended after the JSON
    /// section. The first [`Buffer`] in `subtree.buffers` must have no `uri`
    /// (it refers to this inline chunk); further buffers may be external.
    pub fn write_subtree_binary(
        subtree: &Subtree,
        buffer_data: &[u8],
        opts: WriteOptions,
    ) -> Result<Vec<u8>, crate::Error> {
        let json_bytes = serialize(subtree, opts.pretty_print, "subtree")?;

        // Pad JSON to 8-byte alignment as required by the spec.
        let json_padded_len = (json_bytes.len() + 7) & !7;
        let binary_len = buffer_data.len();

        let mut w = BufferWriter::with_capacity(24 + json_padded_len + binary_len);
        w.write_bytes(b"subt");
        w.write_le(1u32);
        w.write_le(json_padded_len as u64);
        w.write_le(binary_len as u64);
        w.write_bytes(&json_bytes);
        w.align_to(8, 0x20); // pad JSON section with spaces to keep it valid UTF-8
        w.write_bytes(buffer_data);
        debug_assert_eq!(
            w.len(),
            24 + json_padded_len + binary_len,
            "written byte count must match pre-allocated capacity"
        );

        Ok(w.finish())
    }

    /// Build a [`Buffer`] descriptor for the inline binary chunk.
    ///
    /// The returned buffer has no `uri` (inline reference) and `byte_length`
    /// matching the provided data slice. Attach it as `subtree.buffers[0]`
    /// before calling [`write_subtree_binary`](Self::write_subtree_binary).
    pub fn inline_buffer(data: &[u8]) -> Buffer {
        Buffer {
            byte_length: data.len(),
            name: None,
            uri: None,
            data: data.to_vec().into(),
            ..Default::default()
        }
    }
}

/// Serializes a [`Schema`] (3D Tiles metadata schema) to JSON bytes.
pub struct SchemaWriter;

impl SchemaWriter {
    /// Serialize a schema to JSON.
    pub fn write_schema(schema: &Schema, opts: WriteOptions) -> Result<Vec<u8>, crate::Error> {
        serialize(schema, opts.pretty_print, "schema")
    }
}

fn serialize<T: serde::Serialize>(
    value: &T,
    pretty: bool,
    kind: &'static str,
) -> Result<Vec<u8>, crate::Error> {
    let result = if pretty {
        serde_json::to_vec_pretty(value)
    } else {
        serde_json::to_vec(value)
    };
    result.map_err(|error| crate::Error::Serialize {
        kind,
        message: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::from_slice;
    use super::*;
    use crate::generated::{Asset, Availability, BoundingVolume, Class, Refine, Tile};

    fn make_tileset() -> Tileset {
        Tileset {
            asset: Asset {
                version: "1.1".into(),
                ..Default::default()
            },
            geometric_error: 500.0,
            root: Tile {
                bounding_volume: BoundingVolume {
                    sphere: Some([0.0, 0.0, 0.0, 1000.0]),
                    ..Default::default()
                },
                geometric_error: 500.0,
                refine: Some(Refine::Replace),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn tileset_round_trip() {
        let ts = make_tileset();
        let r = TilesetWriter::write_tileset(&ts, WriteOptions::default());
        let bytes = r.unwrap();
        let result = from_slice(&bytes);
        let ts2 = result.expect("round-trip parse failed");
        assert_eq!(&*ts2.asset.version, "1.1");
        assert_eq!(ts2.geometric_error, 500.0);
        assert_eq!(ts2.root.refine, Some(Refine::Replace));
    }

    #[test]
    fn tileset_pretty_print() {
        let ts = make_tileset();
        let r = TilesetWriter::write_tileset(&ts, WriteOptions { pretty_print: true });
        let bytes = r.unwrap();
        assert!(bytes.contains(&b'\n'));
    }

    #[test]
    fn subtree_json_round_trip() {
        let subtree = Subtree {
            tile_availability: Availability {
                constant: Some(1),
                ..Default::default()
            },
            child_subtree_availability: Availability {
                constant: Some(0),
                ..Default::default()
            },
            ..Default::default()
        };
        let r = SubtreeWriter::write_subtree_json(&subtree, WriteOptions::default());
        let bytes = r.unwrap();
        let parsed: Subtree = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed.tile_availability.constant, Some(1));
        assert_eq!(parsed.child_subtree_availability.constant, Some(0));
    }

    #[test]
    fn subtree_binary_envelope_round_trip() {
        let subtree = Subtree {
            tile_availability: Availability {
                constant: Some(1),
                ..Default::default()
            },
            child_subtree_availability: Availability {
                constant: Some(0),
                ..Default::default()
            },
            ..Default::default()
        };
        let payload = vec![0xAAu8, 0xBB, 0xCC, 0xDD];
        let r = SubtreeWriter::write_subtree_binary(&subtree, &payload, WriteOptions::default());
        let bytes = r.unwrap();
        assert_eq!(&bytes[0..4], b"subt");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 1);
        use crate::generated::SubdivisionScheme;
        let av = crate::SubtreeAvailability::from_bytes(&bytes, SubdivisionScheme::Quadtree, 2)
            .expect("should parse");
        assert!(av.is_tile_available_at(0, 0));
    }

    #[test]
    fn schema_round_trip() {
        let schema = Schema {
            id: "test-schema".into(),
            classes: [(
                "Building".into(),
                Class {
                    name: Some("Building".into()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        let r = SchemaWriter::write_schema(&schema, WriteOptions::default());
        let bytes = r.unwrap();
        let parsed: Schema = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(&*parsed.id, "test-schema");
        assert!(parsed.classes.contains_key("Building"));
    }
}
