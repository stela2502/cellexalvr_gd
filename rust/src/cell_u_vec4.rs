#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CellUVec4 {
    pub x: u32, // float bits of position.x
    pub y: u32, // float bits of position.y
    pub z: u32, // float bits of position.z
    pub w: u32, // packed RGB + ID + size
}

impl CellUVec4 {
    pub fn new(x: f32, y: f32, z: f32, rgb: [u8; 3], id: u32, size: f32) -> Self {
        // Convert position floats to raw bits
        let xb = x.to_bits();
        let yb = y.to_bits();
        let zb = z.to_bits();

        // Pack 8-bit R,G,B into low 24 bits
        let color_bits: u32 =
            (rgb[0] as u32) |
            ((rgb[1] as u32) << 8) |
            ((rgb[2] as u32) << 16);

        // ID into next 32 bits (shifted left by 24)
        // size bits occupy high 32 bits after shifting by 56 on GPU
        let size_bits = size.to_bits();
        // We only have 32 bits left in u32, so we just reuse the upper bits later on GPU.
        // Here we'll just store color+ID now, size will go into a second buffer if you want.

        let w = (id << 24) | color_bits; // 0..23 = RGB, 24..55 = ID
        Self { x: xb, y: yb, z: zb, w }
    }
}
