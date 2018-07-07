use std::collections::HashMap;
use std::fs::File;
use std::io::*;

use ::data::*;
use ::color::*;
use ::engine::*;

pub struct PaletteManager<'a> {
    file: &'a File,
    color_cache: GBAColorCache,
    palettes: HashMap<String, Vec<i32>>,
}

impl<'b> PaletteManager<'b> {
    pub fn new(file: &File) -> PaletteManager {
        PaletteManager {
            file,
            color_cache: GBAColorCache::new(),
            palettes: HashMap::new(),
        }
    }

    /// Store the palette of GBA encoded numbers
    pub fn store_palette_i32(&mut self, name: String, colors: Vec<i32>) {
        self.palettes.insert(name, colors);
    }

    /// Convert the color structs to GBA encoded numbers and store them
    pub fn store_palette_colors(&mut self, name: String, colors: Vec<Color>) {
        let gba_colors: Vec<i32> = colors.iter().map(|&c| self.color_cache.to_gba(c)).collect();
        self.store_palette_i32(name, gba_colors);
    }

    /// Load the colors in GBA encoding
    pub fn load_palette_i32<'a>(&'a self, name: String) -> Vec<i32> {
        self.palettes.get(&name).unwrap().clone()
    }

    /// Load the colors as Color structs
    pub fn load_palette_colors(&mut self, name: String) -> Vec<Color> {
        let values: Vec<i32> = self.load_palette_i32(name);
        values.iter().map(|&i| self.color_cache.from_gba(i)).collect()
    }

    /// Read all the palettes in the ROM and store them
    pub fn read_palettes(&mut self) {
        CHARACTERS.iter().for_each(|character| {
            self.read_palette(character);
        });
    }

    /// Read a palette for a specific character and store it
    pub fn read_palette(&mut self, character: &Character) {
        if let Ok(_) = self.file.seek(SeekFrom::Start(character.palette_offset)) {
            let mut colors = [0; 16];
            for i in 0..16 {
                let mut color_buffer = [0; 2];
                self.file.read(&mut color_buffer[..]);

                let a = color_buffer[0] as i32;
                let b = color_buffer[1] as i32;

                // swap the bytes
                let color: i32 = (a << 8) | b;
                colors[i] = color;
                println!("{}#{} = {:x} ({:x}, {:x})", character.name, i, color, color_buffer[0], color_buffer[1]);
            }
            self.store_palette_i32(String::from(character.name), colors.to_vec());

            // TODO remove
            let converted_colors = self.load_palette_colors(character.name.to_string());
            println!("v== {} ==v", character.name);
            for convcol in converted_colors.iter() {
                println!("{:?}", convcol)
            }
            println!("^== {} ==^", character.name)
        } else {
            panic!("Failed to read palette @ ({}) {}", character.name, character.palette_offset);
        }
    }

    /// Write the palette stored for a character into the ROM
    pub fn write_palette(&mut self, character: &Character) {
        if let Ok(_) = self.file.seek(SeekFrom::Start(character.palette_offset)) {
            self.load_palette_i32(character.name.to_string()).iter().for_each(|&i| {
                let a = i & 0xF0;
                let b = i & 0x0F;
                self.file.write(&[a as u8, b as u8]);
            });
        } else {
            panic!("Failed to write palette @ ({}) {}", character.name, character.palette_offset);
        }
    }
}
