use std::fs::File;
use std::io::*;
use std::time::Instant;
use std::result::Result;

use data::*;
use manager::*;

pub struct Engine<'a> {
    pub file: &'a File,
    pub palette_manager: Box<palette::PaletteManager<'a>>,
    pub sprite_manager: Box<sprite::SpriteManager<'a>>,
}

impl<'a> Engine<'a> {
    pub fn new<'b>(file: &'b File) -> Engine<'b> {
        Engine {
            file: &file,
            palette_manager: Box::new(palette::PaletteManager::new(&file)),
            sprite_manager: Box::new(sprite::SpriteManager::new(&file)),
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let engine_timer = Instant::now();
        self.palette_manager.read_palettes();
        println!("Palette ROM loading: {:?}", engine_timer.elapsed());

        for character in CHARACTERS.iter() {
            for i in 0..5 {
                let convert_timer = Instant::now();
                self.sprite_manager.read_sprite(character);
                let mut image = {
                    let spritesheet = self.sprite_manager.load_spritesheet(character)?;
                    let palette = self.palette_manager.load_palette_colors(character.name.to_string());
                    spritesheet.to_img(&palette[..])
                };

                self.sprite_manager.store_image(&mut self.palette_manager, &mut image, character)?;
                self.sprite_manager.write_spritesheet(&mut self.palette_manager, character)?;

                self.sprite_manager.save_spritesheet(&mut self.palette_manager, character);
                println!(" * {} image (iteration {}) conversion: {:?}", character.name, i + 1, convert_timer.elapsed());
            }
        }

        println!("Total Engine Start Time: {:?}", engine_timer.elapsed());
        Ok(())
    }
}
