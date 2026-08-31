//! Tile format detection.

/// Binary tile format detected from the URL or magic bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileFormat {
    /// glTF binary asset.
    Glb,
    /// Batched 3D model (legacy).
    B3dm,
    /// Instanced 3D model (legacy).
    I3dm,
    /// Composite of other tile formats (legacy).
    Cmpt,
    /// Point cloud (legacy).
    Pnts,
    /// JSON payload (a tileset or JSON subtree).
    Json,
    /// Unrecognized format.
    Unknown,
}

impl TileFormat {
    /// Detect the tile format from a URL and/or payload magic bytes.
    ///
    /// Magic bytes take precedence; otherwise the URL's file extension is used.
    pub fn detect(url: &str, data: &[u8]) -> Self {
        if data.len() >= 4 {
            match &data[..4] {
                b"glTF" => return Self::Glb,
                b"b3dm" => return Self::B3dm,
                b"i3dm" => return Self::I3dm,
                b"cmpt" => return Self::Cmpt,
                b"pnts" => return Self::Pnts,
                _ => {}
            }
        }
        let path = url.split('?').next().unwrap_or(url);
        match path
            .rsplit('.')
            .next()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("glb") => Self::Glb,
            Some("b3dm") => Self::B3dm,
            Some("i3dm") => Self::I3dm,
            Some("cmpt") => Self::Cmpt,
            Some("pnts") => Self::Pnts,
            Some("json") => Self::Json,
            _ => Self::Unknown,
        }
    }
}
