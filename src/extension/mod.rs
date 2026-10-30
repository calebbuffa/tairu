mod ext_mesh_features;

pub use ext_mesh_features::*;

use crate::Error;
use serde::{Serialize, de::DeserializeOwned};

/// Trait for types that represent a named 3D Tiles extension payload.
///
/// Reading requires `DeserializeOwned`; writing requires `Serialize`.
/// Presence checks and removal require only this extension identity.
pub trait Extension {
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
    fn extension<E: Extension + DeserializeOwned>(&self) -> Result<Option<E>, Error> {
        match self.extensions().get(E::NAME) {
            Some(value) => E::deserialize(value)
                .map(Some)
                .map_err(|source| Error::json(format!("<extension {}>", E::NAME), source)),
            None => Ok(None),
        }
    }

    /// Returns true when this extension key is present.
    fn has_extension<E: Extension>(&self) -> bool {
        self.extensions().contains_key(E::NAME)
    }

    /// Encode and insert extension payload by type.
    fn set_extension<E: Extension + Serialize>(&mut self, ext: E) -> Result<(), Error> {
        let value = serde_json::to_value(ext)
            .map_err(|source| Error::json(format!("<extension {}>", E::NAME), source))?;
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
    fn extension_capabilities_are_independent() {
        #[derive(serde::Serialize)]
        struct WriteOnly {
            value: i64,
        }
        #[derive(serde::Deserialize)]
        struct ReadOnly {
            value: i64,
        }
        struct IdentityOnly;
        impl Extension for WriteOnly {
            const NAME: &'static str = "TEST_capabilities";
        }
        impl Extension for ReadOnly {
            const NAME: &'static str = WriteOnly::NAME;
        }
        impl Extension for IdentityOnly {
            const NAME: &'static str = WriteOnly::NAME;
        }
        let mut tile = Tile::default();
        tile.set_extension(WriteOnly { value: 42 }).unwrap();
        assert!(tile.has_extension::<IdentityOnly>());
        assert_eq!(tile.extension::<ReadOnly>().unwrap().unwrap().value, 42);
        assert!(tile.remove_extension::<IdentityOnly>());
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
        assert!(matches!(err, Error::Json { .. }));
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
