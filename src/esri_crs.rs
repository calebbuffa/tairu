use crate::{BoundingVolume, Extension};
use serde::{Deserialize, Serialize};

/// ESRI coordinate-reference-system metadata for a tileset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsriCrs {
    /// The ESRI well-known ID of the horizontal coordinate system.
    #[serde(default)]
    pub wkid: u32,

    /// The ESRI well-known ID of the vertical coordinate system, if any.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "vcsWkid")]
    pub vcs_wkid: Option<u32>,

    /// Well-known text definition of the coordinate system (legacy).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wkt: Option<String>,

    /// Well-known text 2 definition of the coordinate system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wkt2: Option<String>,
}

impl EsriCrs {
    /// The EPSG code string, e.g. `"EPSG:4326"` or `"EPSG:4326+EPSG:5703"`.
    pub fn epsg(&self) -> String {
        match self.vcs_wkid {
            Some(vcs) => format!("EPSG:{}+EPSG:{}", self.wkid, vcs),
            None => format!("EPSG:{}", self.wkid),
        }
    }

    /// The WKT definition, preferring `wkt` over `wkt2`.
    pub fn wkt(&self) -> Option<&str> {
        if let Some(wkt) = &self.wkt {
            Some(wkt.as_str())
        } else if let Some(wkt2) = &self.wkt2 {
            Some(wkt2.as_str())
        } else {
            None
        }
    }
}

impl Extension for EsriCrs {
    const NAME: &'static str = "ESRI_crs";
}

impl PartialEq for EsriCrs {
    fn eq(&self, other: &Self) -> bool {
        self.wkid == other.wkid && self.vcs_wkid == other.vcs_wkid
    }
}

impl Eq for EsriCrs {}

/// ESRI coordinate-reference-system transform for a tileset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsriCrsTransform {
    /// The bounding volume the transform applies to, if any.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "boundingVolume"
    )]
    pub bounding_volume: Option<BoundingVolume>,
    /// The column-major coordinate-system transform matrix.
    pub transform: [f64; 16],
}

impl EsriCrsTransform {
    /// Returns `false` when a region bounding volume is set (not supported
    /// for CRS transforms).
    pub fn is_valid(&self) -> bool {
        if let Some(bv) = &self.bounding_volume
            && bv.region.is_some()
        {
            return false;
        }
        true
    }
}

impl Extension for EsriCrsTransform {
    const NAME: &'static str = "ESRI_crs";
}
