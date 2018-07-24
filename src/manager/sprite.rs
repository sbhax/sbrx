extern crate image;

use std::collections::HashMap;
use std::mem;
use std::fs::File;
use std::io::{SeekFrom, Seek, Read, Error, Write};
use std::time::Instant;
use self::image::{ImageBuffer, GenericImage, Rgb};
use self::image::gif::*;

use ::data::*;
use ::color::*;
use ::engine::*;
use ::manager::*;

pub struct Spritesheet {
    pub animations: Vec<Animation>
}

impl Spritesheet {
    pub fn new() -> Spritesheet {
        Spritesheet { animations: Vec::new() }
    }

    pub fn to_img(&self, palette: Vec<Color>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let max_frames = self.animations.iter().map(|animation| animation.frames.len()).max().unwrap();
        let animation_length = self.animations.len();

        let image_width = SECTION_SIZE * FRAME_SIZE * animation_length as usize;
        let image_height = SECTION_SIZE * FRAME_SIZE * max_frames as usize;

        let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(image_width as u32, image_height as u32);

        for (animation_index, animation) in self.animations.iter().enumerate() {
            for (frame_index, frame) in animation.frames.iter().enumerate() {
                for (section_index, section) in frame.sections.iter().enumerate() {
                    for (y, row) in section.bytes.iter().enumerate() {
                        for (x, v) in row.iter().enumerate() {
                            let c = palette[*v as usize];

                            let ix = x + (section_index % FRAME_SIZE) * SECTION_SIZE + (SECTION_SIZE * FRAME_SIZE * animation_index);
                            let iy = y + (section_index / FRAME_SIZE) * SECTION_SIZE + FRAME_SIZE * SECTION_SIZE * frame_index;

                            image.get_pixel_mut(ix as u32, iy as u32).data = [c.r as u8, c.g as u8, c.b as u8];
                        }
                    }
                }
            }
        }
        image
    }
}

pub struct Animation {
    pub frames: Vec<Frame>
}

impl Animation {
    pub fn new() -> Animation {
        Animation { frames: Vec::new() }
    }

    pub fn to_gif(&self, file_name: &str) {
//        let gif_encoder = Encoder::new();
    }
}

// normal character sprites are 6x6 sections
const FRAME_SIZE: usize = 6;

#[derive(Copy, Clone)]
pub struct Frame {
    pub sections: [Section; FRAME_SIZE * FRAME_SIZE]
}

impl Frame {
    pub fn new() -> Frame {
        Frame { sections: [Section::new(); FRAME_SIZE * FRAME_SIZE] }
    }

    pub fn to_image(&self, palette: Vec<Color>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new((FRAME_SIZE * SECTION_SIZE) as u32, (FRAME_SIZE * SECTION_SIZE) as u32);
        for (section_index, section) in self.sections.iter().enumerate() {
            for (y, row) in section.bytes.iter().enumerate() {
                for (x, v) in row.iter().enumerate() {
                    let c = palette[*v as usize];

                    let ix = x + (section_index % FRAME_SIZE) * SECTION_SIZE;
                    let iy = y + (section_index / FRAME_SIZE) * SECTION_SIZE;

                    image.get_pixel_mut(ix as u32, iy as u32).data = [c.r as u8, c.g as u8, c.b as u8];
                }
            }
        }
        image
    }
}

// each section is 8x8 pixels
const SECTION_SIZE: usize = 8;

#[derive(Copy, Clone)]
pub struct Section {
    pub bytes: [[u8; SECTION_SIZE]; SECTION_SIZE]
}

impl Section {
    pub fn new() -> Section {
        Section { bytes: [[0; SECTION_SIZE]; SECTION_SIZE] }
    }
}


// --

pub struct SpriteManager<'a> {
    file: &'a File,
}

impl<'a> SpriteManager<'a> {
    pub fn new<'b>(file: &'b File) -> SpriteManager {
        SpriteManager { file }
    }

    pub fn read_sprite(&mut self, palette_manager: &mut palette::PaletteManager, character: Character) -> Result<(), Error> {
        let start = Instant::now();
        let sprite_data = compute_sprite_offsets(character);

        // used for the image
        let max_frames = sprite_data.iter().max_by_key(|p| { p.1 }).map_or(0, |p| { p.1 });

        let mut spritesheet = Spritesheet::new();

        // go through every animation
        for (animation_index, animation_data) in sprite_data.iter().enumerate() {
            let offset = animation_data.0;
            let frame_count = animation_data.1;

            let mut animation = Animation::new();

            // go through every frame in the animation
            for current_frame in 0..frame_count {
                // read sections from ROM
                let mut frame = Frame::new();
                {
                    /*
                    sort sections
                    00 01 02 03 24 25
                    04 05 06 07 26 27
                    08 09 10 11 28 29
                    12 13 14 15 30 31
                    16 17 18 19 32 33
                    20 21 22 23 34 35
                    */
                    const SECTION_MAPPING: [usize; 36] = [
                        00, 01, 02, 03,
                        06, 07, 08, 09,
                        12, 13, 14, 15,
                        18, 19, 20, 21,
                        24, 25, 26, 27,
                        30, 31, 32, 33,
                        04, 05,
                        10, 11,
                        16, 17,
                        22, 23,
                        28, 29,
                        34, 35,
                    ];


                    let mut current_section = 0;
                    let mut x = 1;
                    let mut y = 1;

                    let frame_offset = offset + FRAME_SIZE as i32 * FRAME_SIZE as i32 * current_frame * 32;
                    let frame_size = FRAME_SIZE as i32 * FRAME_SIZE as i32 * 32;

                    self.file.seek(SeekFrom::Start(frame_offset as u64))?;

                    for i in 0..frame_size {
                        // TODO: optimize reads
                        let mut v = [0; 1];
                        self.file.read(&mut v[..])?;
                        let a = v[0] & 0x0F;
                        let b = (v[0] & 0xF0) >> 4;

                        frame.sections[SECTION_MAPPING[current_section]].bytes[y - 1][x - 1] = a;
                        frame.sections[SECTION_MAPPING[current_section]].bytes[y - 1][x] = b;

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

                animation.frames.push(frame);
            }
            spritesheet.animations.push(animation);
        }

        println!("{} ROM reading took {:?}", character.name, start.elapsed());

        let palette: Vec<Color> = palette_manager.load_palette_colors(character.name.to_string());

        let image_convert_timer = Instant::now();
        let image = spritesheet.to_img(palette);
        println!("{} image conversion took {:?}", character.name, image_convert_timer.elapsed());

        let image_write_timer = Instant::now();
        image.save(format!("roms/sprites/{}.png", character.name))?;
        println!("{} image writing took {:?}", character.name, image_write_timer.elapsed());

        Ok(())
    }
}
