mod esri_crs;
mod ext_mesh_features;

pub use esri_crs::*;
pub use ext_mesh_features::*;

use crate::reader::TileParseError;
use serde::{Serialize, de::DeserializeOwned};

/// Trait for types that represent a named 3D Tiles extension payload.
pub trait Extension: Sized + Serialize + DeserializeOwned {
    /// Extension key in the `extensions` map.
    const NAME: &'static str;
}

/// Typed read/write access to extension data for 3D Tiles model objects.
pub trait HasExtensions {
    /// Returns the raw extension map.
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value>;

    /// Returns the mutable raw extension map.
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value>;

    /// Deserialize extension payload by type.
    fn extension<E: Extension>(&self) -> Result<Option<E>, TileParseError> {
        match self.extensions().get(E::NAME) {
            Some(value) => serde_json::from_value(value.clone())
                .map(Some)
                .map_err(|error| {
                    TileParseError::Validation(format!(
                        "failed to decode extension {}: {}",
                        E::NAME,
                        error
                    ))
                }),
            None => Ok(None),
        }
    }

    /// Returns true when this extension key is present.
    fn has_extension<E: Extension>(&self) -> bool {
        self.extensions().contains_key(E::NAME)
    }

    /// Encode and insert extension payload by type.
    fn set_extension<E: Extension>(&mut self, ext: E) -> Result<(), TileParseError> {
        let value = serde_json::to_value(ext).map_err(|error| {
            TileParseError::Validation(format!("failed to encode extension {}: {}", E::NAME, error))
        })?;
        self.extensions_mut().insert(E::NAME.to_string(), value);
        Ok(())
    }

    /// Remove extension payload by type.
    fn remove_extension<E: Extension>(&mut self) -> bool {
        self.extensions_mut().remove(E::NAME).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::Tile;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestExt {
        value: i64,
    }

    impl Extension for TestExt {
        const NAME: &'static str = "TEST_extension";
    }

    #[test]
    fn typed_round_trip() {
        let mut tile = Tile::default();
        tile.set_extension(TestExt { value: 42 }).unwrap();
        let decoded = tile.extension::<TestExt>().unwrap();
        assert_eq!(decoded, Some(TestExt { value: 42 }));
    }

    #[test]
    fn decode_error_is_validation_error() {
        let mut tile = Tile::default();
        tile.extensions.insert(
            TestExt::NAME.to_string(),
            serde_json::json!({"wrong": true}),
        );
        let err = tile.extension::<TestExt>().expect_err("decode should fail");
        assert!(matches!(err, TileParseError::Validation(_)));
    }

    #[test]
    fn remove_extension_is_idempotent() {
        let mut tile = Tile::default();
        assert!(!tile.remove_extension::<TestExt>());
        tile.set_extension(TestExt { value: 7 }).unwrap();
        assert!(tile.remove_extension::<TestExt>());
        assert!(!tile.remove_extension::<TestExt>());
    }
}
