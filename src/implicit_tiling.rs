use crate::availability::{OctreeTileId, QuadtreeTileId};
use crate::generated::BoundingVolume;

impl QuadtreeTileId {
    /// Resolve an implicit tiling URL template for this tile against `base`.
    pub fn resolve_url(self, base: &str, template: &str) -> String {
        let expanded = expand_tile_url(
            template,
            &[
                ("level", &self.level.to_string()),
                ("x", &self.x.to_string()),
                ("y", &self.y.to_string()),
            ],
        );
        resolve_url(base, &expanded)
    }

    /// Subdivide `root` to the child bounding volume at this tile's coordinates.
    pub fn subdivide_bounding_volume(self, root: &BoundingVolume) -> BoundingVolume {
        let d = level_denominator(self.level);
        let mut out = BoundingVolume::default();
        if let Some(region) = root.region.as_ref() {
            let (w, s, e, n, minh, maxh) = (
                region[0], region[1], region[2], region[3], region[4], region[5],
            );
            let (lon, lat) = ((e - w) / d, (n - s) / d);
            let (tx, ty) = (self.x as f64, self.y as f64);
            out.region = Some([
                w + lon * tx,
                s + lat * ty,
                w + lon * (tx + 1.0),
                s + lat * (ty + 1.0),
                minh,
                maxh,
            ]);
            return out;
        }
        if let Some(box_values) = root.r#box.as_ref() {
            out.r#box = Some(subdivide_obb_quad(box_values, self.x, self.y, d));
            return out;
        }
        if let Some(sphere) = root.sphere.as_ref() {
            // conservative: bound the OBB child as a sphere
            let fake_box = sphere_to_isotropic_box(sphere);
            let child_box = subdivide_obb_quad(&fake_box, self.x, self.y, d);
            out.sphere = Some(obb_bounding_sphere(&child_box));
            return out;
        }
        out
    }
}

impl OctreeTileId {
    /// Resolve an implicit tiling URL template for this tile against `base`.
    pub fn resolve_url(self, base: &str, template: &str) -> String {
        let expanded = expand_tile_url(
            template,
            &[
                ("level", &self.level.to_string()),
                ("x", &self.x.to_string()),
                ("y", &self.y.to_string()),
                ("z", &self.z.to_string()),
            ],
        );
        resolve_url(base, &expanded)
    }

    /// Subdivide `root` to the child bounding volume at this tile's coordinates.
    pub fn subdivide_bounding_volume(self, root: &BoundingVolume) -> BoundingVolume {
        let d = level_denominator(self.level);
        let mut out = BoundingVolume::default();
        if let Some(region) = root.region.as_ref() {
            let (w, s, e, n, minh, maxh) = (
                region[0], region[1], region[2], region[3], region[4], region[5],
            );
            let (lon, lat, h) = ((e - w) / d, (n - s) / d, (maxh - minh) / d);
            let (tx, ty, tz) = (self.x as f64, self.y as f64, self.z as f64);
            out.region = Some([
                w + lon * tx,
                s + lat * ty,
                w + lon * (tx + 1.0),
                s + lat * (ty + 1.0),
                minh + h * tz,
                minh + h * (tz + 1.0),
            ]);
            return out;
        }
        if let Some(box_values) = root.r#box.as_ref() {
            out.r#box = Some(subdivide_obb_oct(box_values, self.x, self.y, self.z, d));
            return out;
        }
        if let Some(sphere) = root.sphere.as_ref() {
            let fake_box = sphere_to_isotropic_box(sphere);
            let child_box = subdivide_obb_oct(&fake_box, self.x, self.y, self.z, d);
            out.sphere = Some(obb_bounding_sphere(&child_box));
            return out;
        }
        out
    }
}

// 2^level
fn level_denominator(level: u32) -> f64 {
    (1u64 << level) as f64
}

// 3D Tiles `box` layout: [cx, cy, cz, xhx, xhy, xhz, yhx, yhy, yhz, zhx, zhy, zhz]
fn subdivide_obb_quad(b: &[f64], tx: u32, ty: u32, d: f64) -> [f64; 12] {
    use glam::DVec3;
    let center = DVec3::new(b[0], b[1], b[2]);
    let xh = DVec3::new(b[3], b[4], b[5]);
    let yh = DVec3::new(b[6], b[7], b[8]);
    let zh = DVec3::new(b[9], b[10], b[11]);
    let xd = xh * 2.0 / d;
    let yd = yh * 2.0 / d;
    let min = center - xh - yh - zh;
    let child_center = min + xd * (tx as f64 + 0.5) + yd * (ty as f64 + 0.5) + zh;
    let new_xh = xd * 0.5;
    let new_yh = yd * 0.5;
    [
        child_center.x,
        child_center.y,
        child_center.z,
        new_xh.x,
        new_xh.y,
        new_xh.z,
        new_yh.x,
        new_yh.y,
        new_yh.z,
        zh.x,
        zh.y,
        zh.z,
    ]
}

fn subdivide_obb_oct(b: &[f64], tx: u32, ty: u32, tz: u32, d: f64) -> [f64; 12] {
    use glam::DVec3;
    let center = DVec3::new(b[0], b[1], b[2]);
    let xh = DVec3::new(b[3], b[4], b[5]);
    let yh = DVec3::new(b[6], b[7], b[8]);
    let zh = DVec3::new(b[9], b[10], b[11]);
    let xd = xh * 2.0 / d;
    let yd = yh * 2.0 / d;
    let zd = zh * 2.0 / d;
    let min = center - xh - yh - zh;
    let child_center =
        min + xd * (tx as f64 + 0.5) + yd * (ty as f64 + 0.5) + zd * (tz as f64 + 0.5);
    let (new_xh, new_yh, new_zh) = (xd * 0.5, yd * 0.5, zd * 0.5);
    [
        child_center.x,
        child_center.y,
        child_center.z,
        new_xh.x,
        new_xh.y,
        new_xh.z,
        new_yh.x,
        new_yh.y,
        new_yh.z,
        new_zh.x,
        new_zh.y,
        new_zh.z,
    ]
}

// Treat a sphere (cx,cy,cz,r) as an axis-aligned box with half-axes r along each axis.
fn sphere_to_isotropic_box(s: &[f64]) -> [f64; 12] {
    let r = s[3];
    [s[0], s[1], s[2], r, 0.0, 0.0, 0.0, r, 0.0, 0.0, 0.0, r]
}

// Minimum bounding sphere of an OBB: center at OBB center, radius = length of half-diagonal.
fn obb_bounding_sphere(b: &[f64]) -> [f64; 4] {
    use glam::DVec3;
    let xh = DVec3::new(b[3], b[4], b[5]);
    let yh = DVec3::new(b[6], b[7], b[8]);
    let zh = DVec3::new(b[9], b[10], b[11]);
    let r = (xh + yh + zh).length();
    [b[0], b[1], b[2], r]
}

fn expand_tile_url(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

pub fn resolve_url(base: &str, path: &str) -> String {
    crate::uri::resolve_uri(base, path)
}
#[cfg(test)]
mod tests {
    use crate::availability::{OctreeTileId, QuadtreeTileId};

    #[test]
    fn test_resolve_url_quad() {
        let tile = QuadtreeTileId::new(3, 5, 2);
        let url = tile.resolve_url(
            "https://example.com/tileset/tileset.json",
            "subtrees/{level}/{x}/{y}.subtree",
        );
        assert_eq!(url, "https://example.com/tileset/subtrees/3/5/2.subtree");
    }

    #[test]
    fn test_resolve_url_oct() {
        let tile = OctreeTileId::new(2, 1, 3, 0);
        let url = tile.resolve_url(
            "https://example.com/tileset.json",
            "subtrees/{level}/{x}/{y}/{z}.subtree",
        );
        assert_eq!(url, "https://example.com/subtrees/2/1/3/0.subtree");
    }

    #[test]
    fn test_resolve_url_absolute_passthrough() {
        let tile = QuadtreeTileId::new(0, 0, 0);
        let url = tile.resolve_url(
            "https://example.com/tileset.json",
            "https://cdn.example.com/subtrees/{level}/{x}/{y}.subtree",
        );
        assert!(url.starts_with("https://cdn.example.com/"));
    }
}
