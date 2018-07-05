use std::collections::HashMap;
use std::fs::File;

use ::color::*;
use ::engine::*;

pub struct PaletteManager {
    //    engine: &'a Engine<'a>,
    color_cache: GBAColorCache,
    palettes: HashMap<String, Vec<i32>>,
}

impl PaletteManager {
    pub fn new() -> PaletteManager {
        PaletteManager {
            color_cache: GBAColorCache::new(),
            palettes: HashMap::new(),
        }
    }

    pub fn store_palette_i32(mut self, name: String, colors: Vec<i32>) {
        self.palettes.insert(name, colors);
    }

    pub fn store_palette_colors(mut self, name: String, colors: Vec<Color>) {
        let gba_colors: Vec<i32> = colors.iter().map(|&c| self.color_cache.to_gba(c)).collect();
        self.store_palette_i32(name, gba_colors);
    }

    pub fn load_palette_i32<'a>(&'a self, name: String) -> Vec<i32> {
        self.palettes.get(&name).unwrap().clone()
    }

    pub fn load_palette_colors(&mut self, name: String) -> Vec<Color> {
        let values: Vec<i32> = self.load_palette_i32(name);
        values.iter().map(|&i| self.color_cache.from_gba(i)).collect()
    }
}
