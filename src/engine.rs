use std::fs::File;
use std::io::*;
use std::time::Instant;
use std::result::Result;
use std::sync::{Arc, Mutex};

use data::*;
use manager::*;

pub struct Engine {
    pub file: Arc<Mutex<File>>,
    pub palette_manager: Box<palette::PaletteManager>,
    pub sprite_manager: Box<sprite::SpriteManager>,
}

impl Engine {
    pub fn new(file: Arc<Mutex<File>>) -> Engine {
        Engine {
            file: file.clone(),
            palette_manager: Box::new(palette::PaletteManager::new(file.clone())),
            sprite_manager: Box::new(sprite::SpriteManager::new(file.clone())),
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let engine_timer = Instant::now();
        self.palette_manager.read_palettes()?;
        println!("Palette ROM loading: {:?}", engine_timer.elapsed());
        self.sprite_manager.read_sprites()?;
        println!("Sprite ROM loading: {:?}", engine_timer.elapsed());
        Ok(())
    }
}
