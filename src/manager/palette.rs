use std::collections::HashMap;
use std::fs::File;
use std::io::{SeekFrom, Seek, Read, Error, Write};
use std::result::Result;

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
        let gba_colors: Vec<i32> = colors.iter().map(|&c| self.color_cache.rgb_to_gba(c)).collect();
        self.store_palette_i32(name, gba_colors);
    }

    /// Load the colors in GBA encoding
    pub fn load_palette_i32<'a>(&'a self, name: String) -> Vec<i32> {
        self.palettes.get(&name).unwrap().clone()
    }

    /// Load the colors as Color structs
    pub fn load_palette_colors(&mut self, name: String) -> Vec<Color> {
        let values: Vec<i32> = self.load_palette_i32(name);
        values.iter().map(|&i| self.color_cache.gba_to_rgb(i)).collect()
    }

    /// Read all the palettes in the ROM and store them
    pub fn read_palettes(&mut self) -> Result<(), Error> {
        for character in CHARACTERS.iter() {
            self.read_palette(character)?
        }
        Ok(())
    }

    /// Read a palette for a specific character and store it
    pub fn read_palette(&mut self, character: &Character) -> Result<(), Error> {
        self.file.seek(SeekFrom::Start(character.palette_offset))?;
        let mut colors = [0; 16];
        for i in 0..16 {
            let mut color_buffer: [u8; 2] = [0; 2];
            let result = self.file.read(&mut color_buffer[..])?;

            let a = color_buffer[0] as i32;
            let b = color_buffer[1] as i32;

            // swap the bytes
            let color: i32 = (b << 8) | a;
            colors[i] = color;
//            println!("{}#{} = {:x} ({:x}, {:x})", character.name, i, color, a, b);
        }
        self.store_palette_i32(String::from(character.name), colors.to_vec());

//        // TODO remove
//        let converted_colors = self.load_palette_colors(character.name.to_string());
//        println!("v== {} ==v", character.name);
//        for convcol in converted_colors.iter() {
//            println!("{:?}", convcol)
//        }
//        println!("^== {} ==^", character.name);
        Ok(())
    }

    /// Write the palette stored for a character into the ROM
    pub fn write_palette(&mut self, character: &Character) -> Result<(), Error> {
        self.file.seek(SeekFrom::Start(character.palette_offset))?;
        for i in self.load_palette_i32(character.name.to_string()).iter() {
            let a = i & 0xF0;
            let b = i & 0x0F;
            match self.file.write(&[a as u8, b as u8]) {
                Ok(_) => {}
                Err(error) => {
                    return Err(error);
                }
            }
        }
        Ok(())
    }
}
