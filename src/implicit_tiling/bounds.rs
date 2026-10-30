use super::{OctreeTileId, QuadtreeTileId};
use crate::generated::BoundingVolume;

impl QuadtreeTileId {
    /// Expands an implicit tiling URL template for this tile.
    ///
    /// The result is relative to the declaring tileset unless the template is
    /// absolute; resolving it is the transport layer's concern.
    pub fn expand_url(self, template: &str) -> String {
        expand_tile_url(
            template,
            &[
                ("level", &self.level.to_string()),
                ("x", &self.x.to_string()),
                ("y", &self.y.to_string()),
            ],
        )
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
        out
    }
}

impl OctreeTileId {
    /// Expands an implicit tiling URL template for this tile.
    ///
    /// The result is relative to the declaring tileset unless the template is
    /// absolute; resolving it is the transport layer's concern.
    pub fn expand_url(self, template: &str) -> String {
        expand_tile_url(
            template,
            &[
                ("level", &self.level.to_string()),
                ("x", &self.x.to_string()),
                ("y", &self.y.to_string()),
                ("z", &self.z.to_string()),
            ],
        )
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
        out
    }
}

// 2^level
fn level_denominator(level: u32) -> f64 {
    (1u64 << level) as f64
}

#[derive(Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }
}

// 3D Tiles `box` layout: [cx, cy, cz, xhx, xhy, xhz, yhx, yhy, yhz, zhx, zhy, zhz]
fn subdivide_obb_quad(b: &[f64], tx: u32, ty: u32, d: f64) -> [f64; 12] {
    let center = Vec3::new(b[0], b[1], b[2]);
    let xh = Vec3::new(b[3], b[4], b[5]);
    let yh = Vec3::new(b[6], b[7], b[8]);
    let zh = Vec3::new(b[9], b[10], b[11]);
    let xd = xh.scale(2.0 / d);
    let yd = yh.scale(2.0 / d);
    let min = center.sub(xh).sub(yh).sub(zh);
    let child_center = min
        .add(xd.scale(tx as f64 + 0.5))
        .add(yd.scale(ty as f64 + 0.5))
        .add(zh);
    let new_xh = xd.scale(0.5);
    let new_yh = yd.scale(0.5);
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
    let center = Vec3::new(b[0], b[1], b[2]);
    let xh = Vec3::new(b[3], b[4], b[5]);
    let yh = Vec3::new(b[6], b[7], b[8]);
    let zh = Vec3::new(b[9], b[10], b[11]);
    let xd = xh.scale(2.0 / d);
    let yd = yh.scale(2.0 / d);
    let zd = zh.scale(2.0 / d);
    let min = center.sub(xh).sub(yh).sub(zh);
    let child_center = min
        .add(xd.scale(tx as f64 + 0.5))
        .add(yd.scale(ty as f64 + 0.5))
        .add(zd.scale(tz as f64 + 0.5));
    let (new_xh, new_yh, new_zh) = (xd.scale(0.5), yd.scale(0.5), zd.scale(0.5));
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

fn expand_tile_url(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{OctreeTileId, QuadtreeTileId};

    #[test]
    fn expands_quadtree_template() {
        let url = QuadtreeTileId::new(3, 5, 2).expand_url("subtrees/{level}/{x}/{y}.subtree");
        assert_eq!(url, "subtrees/3/5/2.subtree");
    }

    #[test]
    fn expands_octree_template() {
        let url = OctreeTileId::new(2, 1, 3, 0).expand_url("subtrees/{level}/{x}/{y}/{z}.subtree");
        assert_eq!(url, "subtrees/2/1/3/0.subtree");
    }
}
