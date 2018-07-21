extern crate image;

use std::collections::HashMap;
use std::mem;
use std::fs::File;
use std::io::{SeekFrom, Seek, Read, Error, Write};
use self::image::{ImageBuffer, GenericImage, Rgb};

use ::data::*;
use ::color::*;
use ::engine::*;
use ::manager::*;

pub struct SpriteManager<'a> {
    file: &'a File,
}

impl<'a> SpriteManager<'a> {
    pub fn new<'b>(file: &'b File) -> SpriteManager {
        SpriteManager { file }
    }

    pub fn read_sprite(&mut self, palette_manager: &mut palette::PaletteManager, character: Character) -> Result<(), Error> {
        // normal character sprites are 6x6 sections
        const SIZE: usize = 6;
        // each section is 8x8 pixels
        const SECTION_SIZE: usize = 8;
        let sprite_data = compute_sprite_offsets(character);

        // used for the image
        let max_frames = sprite_data.iter().max_by_key(|p| { p.1 }).map_or(0, |p| { p.1 });

        let mut animations: Vec<Vec<Vec<Vec<Vec<u8>>>>> = Vec::with_capacity(sprite_data.len());
        unsafe {
            animations.set_len(sprite_data.len());
        }

        // go through every animation
        let mut animation_index = 0;
        for animation_data in sprite_data.iter() {
            let offset = animation_data.0;
            let frame_count = animation_data.1;

            let mut frames: Vec<Vec<Vec<Vec<u8>>>> = vec![vec![vec![vec![0; SECTION_SIZE]; SECTION_SIZE]; SIZE * SIZE]; frame_count as usize];

            // go through every frame in the animation
            for current_frame in 0..frame_count {
                // read sections from ROM
                let mut sections: Vec<Vec<Vec<u8>>> = vec![vec![vec![0; SECTION_SIZE]; SECTION_SIZE]; SIZE * SIZE];

                {
                    let mut current_section = 0;
                    let mut x = 1;
                    let mut y = 1;

                    let frame_offset = offset + SIZE as i32 * SIZE as i32 * current_frame * 32;
                    let frame_size = SIZE as i32 * SIZE as i32 * 32;

                    self.file.seek(SeekFrom::Start(frame_offset as u64))?;

                    for i in 0..frame_size {
                        // TODO: optimize reads
                        let mut v = [0; 1];
                        self.file.read(&mut v[..])?;
                        let a = v[0] & 0x0F;
                        let b = (v[0] & 0xF0) >> 4;

                        sections[current_section][y - 1][x - 1] = a;
                        sections[current_section][y - 1][x] = b;

                        // check bounds
                        x += 1;
                        if x != 0 && x % 8 == 0 {
                            x -= 8;
                            if y != 0 && y % 8 == 0 {
                                x = 0;
                                y = 0;
                                current_section += 1;
                            }
                            y += 1;
                        }
                        x += 1;
                    }
                }

                /*
                sort sections
                00 01 02 03 24 25
                04 05 06 07 26 27
                08 09 10 11 28 29
                12 13 14 15 30 31
                16 17 18 19 32 33
                20 21 22 23 34 35
                */

                // JESUS CHRIST!!!! KILL IT!!! KILL IT NOW!!!!
                // HOW COULD YOU LET THIS HAPPEN!?!??!?
                frames[current_frame as usize] = vec![
                    sections[00].clone(), sections[01].clone(), sections[02].clone(), sections[03].clone(), sections[24].clone(), sections[25].clone(),
                    sections[04].clone(), sections[05].clone(), sections[06].clone(), sections[07].clone(), sections[26].clone(), sections[27].clone(),
                    sections[08].clone(), sections[09].clone(), sections[10].clone(), sections[11].clone(), sections[28].clone(), sections[29].clone(),
                    sections[12].clone(), sections[13].clone(), sections[14].clone(), sections[15].clone(), sections[30].clone(), sections[31].clone(),
                    sections[16].clone(), sections[17].clone(), sections[18].clone(), sections[19].clone(), sections[32].clone(), sections[33].clone(),
                    sections[20].clone(), sections[21].clone(), sections[22].clone(), sections[23].clone(), sections[34].clone(), sections[35].clone(),
                ];
            }

            animations[animation_index] = frames;

            animation_index += 1;
        }

        let palette: Vec<Color> = palette_manager.load_palette_colors(character.name.to_string());

        // create the spritesheet
        let image_width = SECTION_SIZE * SIZE * sprite_data.len() as usize;
        let image_height = SECTION_SIZE * SIZE * max_frames as usize;
        let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(image_width as u32, image_height as u32);

        // write the animations to the image

        let animation_length = animations.len();

        for (animation_index, animation) in animations.iter().enumerate() {
            for (frame_index, frame) in animation.iter().enumerate() {
                for (section_index, section) in frame.iter().enumerate() {
                    for (y, row) in section.iter().enumerate() {
                        for (x, v) in row.iter().enumerate() {
                            let c = palette[*v as usize];

                            let ix = x + (section_index % SIZE) * SECTION_SIZE + (SECTION_SIZE * SIZE * animation_index);
                            let iy = y + (section_index / SIZE) * SECTION_SIZE + SIZE * SECTION_SIZE * frame_index;

                            image.get_pixel_mut(ix as u32, iy as u32).data = [c.r as u8, c.g as u8, c.b as u8];
                        }
                    }
                }
            }
        }

        image.save(format!("roms/sprites/{}.png", character.name))?;
        Ok(())
    }
}
