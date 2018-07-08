use std::collections::HashMap;
use std::fs::File;
use std::io::{SeekFrom, Seek, Read, Error, Write};

use ::data::*;
use ::color::*;
use ::engine::*;

pub struct SpriteManager<'a> {
    file: &'a File,
}

impl<'a> SpriteManager<'a> {
    pub fn new<'b>(file: &'b File) -> SpriteManager {
        SpriteManager { file }
    }

    pub fn read_sprite(&mut self, character: Character) -> Result<(), Error> {
        // normal character sprites are 6x6 sections
        let size = 6;
        let sprite_data = compute_sprite_offsets(character);
        // used for the image
        // let max_frames = sprite_data.iter().max_by_key(|p| { p.1 });

        // go through every animation
        let mut animation_index = 0;
        for animation_data in sprite_data.iter() {
            let offset = animation_data.0;
            let frame_count = animation_data.1;

            // go through every frame in the animation
            for current_frame in 0..frame_count {
                // read sections from ROM
                {
                    let mut current_section = 0;
                    let mut x = 0;
                    let mut y = 0;

                    let frame_offset = offset + size * size * current_frame * 32;
                    let frame_size = frame_offset + size * size * 32;
                    self.file.seek(SeekFrom::Start(frame_offset as u64))?;

                    for i in 0..frame_size {
                        // TODO: optimize reads
                        let mut v = [0; 1];
                        self.file.read(&mut v[..])?;
                        let a = v[0] & 0x0F;
                        let b = (v[0] & 0xF0) >> 4;

                        // sections[currentSection].values[y - 1][x - 1] = v1;
                        // sections[currentSection].values[y - 1][x] = v2;

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

                // sort sections
            }

            animation_index += 1;
        }
        Ok(())
    }
}
