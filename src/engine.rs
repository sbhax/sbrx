use std::fs::File;

use manager::*;

pub struct Engine {
    file: File,
    palette_manager: Box<palette::PaletteManager>,
}

impl<'a> Engine {
    pub fn new(file: File) -> Engine {
        Engine {
            file,
            palette_manager: Box::new(palette::PaletteManager::new()),
        }
    }

    pub fn start(&mut self) {}
}
