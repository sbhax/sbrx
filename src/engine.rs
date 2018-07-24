use std::fs::File;
use std::io::*;
use std::time::Instant;

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

    pub fn start(&mut self) {
        let engine_timer = Instant::now();
        self.palette_manager.read_palettes();
        println!("Palette ROM loading: {:?}", engine_timer.elapsed());
        self.sprite_manager.read_sprites();
        println!("Total Engine Start Time: {:?}", engine_timer.elapsed());
    }
}
