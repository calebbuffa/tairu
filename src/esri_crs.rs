use crate::{BoundingVolume, Extension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsriCrs {
    #[serde(default)]
    pub wkid: u32,

    #[serde(default, skip_serializing_if = "Option::is_none", rename = "vcsWkid")]
    pub vcs_wkid: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wkt: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wkt2: Option<String>,
}

impl EsriCrs {
    pub fn epsg(&self) -> String {
        match self.vcs_wkid {
            Some(vcs) => format!("EPSG:{}+EPSG:{}", self.wkid, vcs),
            None => format!("EPSG:{}", self.wkid),
        }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsriCrsTransform {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "boundingVolume"
    )]
    pub bounding_volume: Option<BoundingVolume>,
    pub transform: [f64; 16],
}

impl EsriCrsTransform {
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
