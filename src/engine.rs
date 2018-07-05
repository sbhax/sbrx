use std::fs::File;
use std::io::*;

use data::*;
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

    pub fn start(&mut self) {
        self.load_palettes()
    }

    pub fn load_palettes(&mut self) {
        CHARACTERS.iter().for_each(|character| {
            if let Ok(_) = self.file.seek(SeekFrom::Start(character.palette_offset)) {
                let mut colors = [0; 16];
                for i in 0..16 {
                    let mut color_buffer = [0; 2];
                    self.file.read(&mut color_buffer[..]);

                    let a = color_buffer[0] as i32;
                    let b = color_buffer[1] as i32;

                    let color: i32 = (a << 8) | b;
                    colors[i] = color;
                    println!("{}#{} = {:x} ({:x}, {:x})", character.name, i, color, color_buffer[0], color_buffer[1]);
                }
                self.palette_manager.store_palette_i32(String::from(character.name), colors.to_vec());

                // TODO remove
                let converted_colors = self.palette_manager.load_palette_colors(character.name.to_string());
                println!("v== {} ==v", character.name);
                for convcol in converted_colors.iter() {
                    println!("{:?}", convcol)
                }
                println!("^== {} ==^", character.name)
            } else {
                panic!("Failed to read palette @ ({}) {}", character.name, character.palette_offset);
            }
        });
    }
}
