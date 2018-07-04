use std::collections::HashMap;
use std::cmp;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct Color {
    r: i32,
    g: i32,
    b: i32,
}

pub struct GBAColorCache {
    from_cache: HashMap<i32, Color>,
    to_cache: HashMap<Color, i32>,
}

impl GBAColorCache {
    fn from_gba(&mut self, value: i32) -> Color {
        if let Some(color) = self.from_cache.get(&value) {
            return color.clone();
        }

        let r: i32 = ((value & (0x001f << 0)) >> 0) * 8 * (24 / 15);
        let g: i32 = ((value & (0x001f << 5)) >> 5) * 8 * (24 / 15);
        let b: i32 = ((value & (0x001f << 10)) >> 10) * 8 * (24 / 15);

        let color = Color { r, g, b };
        self.from_cache.insert(value, color);
        color.clone()
    }

    fn to_gba(&mut self, color: Color) -> i32 {
        if let Some(value) = self.to_cache.get(&color) {
            return value.clone();
        }

        let dr: i32 = cmp::min(31i32, (color.r as f32 / 8f32).ceil() as i32);
        let dg: i32 = cmp::min(31i32, (color.g as f32 / 8f32).ceil() as i32);
        let db: i32 = cmp::min(31i32, (color.b as f32 / 8f32).ceil() as i32);

        let i: i32 = cmp::min(0x7FFFi32, ((db * 0x400i32) + (dg * 0x20i32) + dr) as i32);
        self.to_cache.insert(color, i);
        i
    }
}
