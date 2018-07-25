extern crate image;

use std::collections::HashMap;
use std::mem;
use std::fs::File;
use std::io::{SeekFrom, Seek, Read, Error, ErrorKind, Write};
use std::time::Instant;
use std::sync::{Arc, Mutex};
use self::image::{ImageBuffer, GenericImage, Rgb};
use self::image::gif::*;

use ::data::*;
use ::color::*;
use ::engine::*;
use ::manager::*;

// colors used for the background in spritesheets
const PURPLE_1: Color = Color { r: 255, g: 0, b: 255 };
const PURPLE_2: Color = Color { r: 185, g: 0, b: 255 };
const PURPLE_3: Color = Color { r: 185, g: 0, b: 185 }; // no frame

/// normal character sprites are 6x6 sections
pub const FRAME_SIZE: usize = 6;

/// each section is 8x8 pixels
pub const SECTION_SIZE: usize = 8;

pub struct Spritesheet {
    pub animations: Vec<Animation>
}

impl Spritesheet {
    pub fn new() -> Spritesheet {
        Spritesheet { animations: Vec::new() }
    }

    /// convert a spritesheet to an image
    pub fn to_img(&self, palette: &[Color]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let max_frames = self.animations.iter().map(|animation| animation.frames.len()).max().unwrap();
        let animation_length = self.animations.len();

        let image_width = SECTION_SIZE * FRAME_SIZE * animation_length as usize;
        let image_height = SECTION_SIZE * FRAME_SIZE * max_frames as usize;

        let purple_rgb = Rgb { data: [PURPLE_3.r as u8, PURPLE_3.g as u8, PURPLE_3.b as u8] };
        let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(image_width as u32, image_height as u32, purple_rgb);

        for (animation_index, animation) in self.animations.iter().enumerate() {
            for (frame_index, frame) in animation.frames.iter().enumerate() {
                for (section_index, section) in frame.sections.iter().enumerate() {
                    for (y, row) in section.bytes.iter().enumerate() {
                        for (x, v) in row.iter().enumerate() {
                            let c: Color;
                            let b = *v as usize;

                            let c = if b == 0 {
                                if (animation_index % 2 == 0) == (frame_index % 2 == 0) {
                                    PURPLE_1
                                } else {
                                    PURPLE_2
                                }
                            } else {
                                palette[b]
                            };

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

    /// convert an image to a spritesheet
    pub fn from_img(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, character: &Character) -> Result<(Spritesheet, Vec<Color>), Error> {
        let mut spritesheet = Spritesheet::new();
        let mut palette = vec![Color { r: 0, g: 248, b: 248 }];

        for (animation_index, frames) in character.sprite_frames.iter().enumerate() {
            let mut animation = Animation::new();
            spritesheet.animations.push(animation);
            for frame_index in 0..*frames {
                let mut frame = Frame::new();
                spritesheet.animations[animation_index].frames.push(frame);
                for sy in 0..FRAME_SIZE {
                    for sx in 0..FRAME_SIZE {
                        for y in 0..SECTION_SIZE {
                            for x in 0..SECTION_SIZE {
                                let ix = sx * SECTION_SIZE + x + (SECTION_SIZE * FRAME_SIZE * animation_index as usize);
                                let iy = sy * SECTION_SIZE + y + (SECTION_SIZE * FRAME_SIZE * frame_index as usize);
                                let section_index = sx + sy * FRAME_SIZE;

                                // convert to our color struct
                                let rgb = image.get_pixel_mut(ix as u32, iy as u32).data;
                                let color = Color { r: rgb[0] as i32, g: rgb[1] as i32, b: rgb[2] as i32 };

                                let mut color_index;

                                if color == PURPLE_1 || color == PURPLE_2 || color == PURPLE_3 {
                                    color_index = 0;
                                } else if palette.contains(&color) {
                                    color_index = palette.iter().position(|&c| c == color).unwrap();
                                } else if palette.len() < 16 {
                                    palette.push(color);
                                    color_index = palette.len() - 1;
                                } else {
                                    return Err(Error::new(ErrorKind::InvalidData, format!("Invalid color found at {}, {}", ix, iy)));
                                }

                                spritesheet
                                    .animations[animation_index]
                                    .frames[frame_index as usize]
                                    .sections[section_index]
                                    .bytes[y][x] = color_index as u8;
                            }
                        }
                    }
                }
            }
        }

        while palette.len() < 16 {
            palette.push(Color { r: 0, g: 0, b: 0 });
        }

        Ok((spritesheet, palette))
    }
}

pub struct Animation {
    pub frames: Vec<Frame>
}

impl Animation {
    pub fn new() -> Animation {
        Animation { frames: Vec::new() }
    }

    pub fn get_frames(&self, palette: &[Color]) -> Vec<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        self.frames.iter().map(|frame| frame.to_image(palette)).collect()
    }
}

#[derive(Copy, Clone)]
pub struct Frame {
    pub sections: [Section; FRAME_SIZE * FRAME_SIZE]
}

impl Frame {
    pub fn new() -> Frame {
        Frame { sections: [Section::new(); FRAME_SIZE * FRAME_SIZE] }
    }

    pub fn to_image(&self, palette: &[Color]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(
            (FRAME_SIZE * SECTION_SIZE) as u32,
            (FRAME_SIZE * SECTION_SIZE) as u32,
        );

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

pub struct SpriteManager {
    file: Arc<Mutex<File>>,
    pub spritesheets: HashMap<String, Spritesheet>,
}

impl SpriteManager {
    pub fn new(file: Arc<Mutex<File>>) -> SpriteManager {
        SpriteManager {
            file: file.clone(),
            spritesheets: HashMap::new()
        }
    }

    pub fn read_sprites(&mut self) -> Result<(), Error> {
        for character in CHARACTERS.iter() {
            self.read_sprite(character)?;
        }
        Ok(())
    }

    pub fn read_sprite(&mut self, character: &Character) -> Result<(), Error> {
        let spritesheet = self.read_spritesheet_from_rom(character)?;
        self.spritesheets.insert(character.name.to_string(), spritesheet);
        Ok(())
    }

    pub fn read_spritesheet_from_rom(&mut self, character: &Character) -> Result<Spritesheet, Error> {
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

                    const FRAME_BYTE_COUNT: usize = FRAME_SIZE * FRAME_SIZE * 32;
                    let frame_offset = offset + FRAME_BYTE_COUNT as i32 * current_frame;

                    self.file.lock().unwrap().seek(SeekFrom::Start(frame_offset as u64))?;

                    let mut buffer = [0; FRAME_BYTE_COUNT];
                    self.file.lock().unwrap().read(&mut buffer[..])?;

                    for i in 0..FRAME_BYTE_COUNT {
                        let a = buffer[i] & 0x0F;
                        let b = (buffer[i] & 0xF0) >> 4;

                        frame.sections[SECTION_MAPPING[current_section]].bytes[y - 1][x - 1] = a;
                        frame.sections[SECTION_MAPPING[current_section]].bytes[y - 1][x] = b;

                        // check bounds
                        x += 1;
                        if x % 8 == 0 {
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
        println!(" * {} ROM reading: {:?}", character.name, start.elapsed());
        Ok(spritesheet)
    }

    pub fn store_image(&mut self, palette_manager: &mut palette::PaletteManager, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, character: &Character) -> Result<(), Error> {
        let (spritesheet, palette) = Spritesheet::from_img(image, character)?;
        self.spritesheets.insert(character.name.to_string(), spritesheet);
        palette_manager.store_palette_colors(character.name.to_string(), palette);
        Ok(())
    }

    pub fn write_spritesheets(&mut self, palette_manager: &mut palette::PaletteManager) -> Result<(), Error> {
        for character in CHARACTERS.iter() {
            self.write_spritesheet(palette_manager, character)?;
        }
        Ok(())
    }

    pub fn write_spritesheet(&mut self, palette_manager: &mut palette::PaletteManager, character: &Character) -> Result<(), Error> {
        let spritesheet_o = self.spritesheets.get(&character.name.to_string());
        palette_manager.write_palette(character)?;
        if let Some(spritesheet) = spritesheet_o {
            let mut bytes: Vec<u8> = Vec::new();

            for animation in spritesheet.animations.iter() {
                for frame in animation.frames.iter() {
                    // resort the sections back to the GBA format
                    let mut sorted_sections = [Section::new(); FRAME_SIZE * FRAME_SIZE];

                    const SECTION_MAPPING: [usize; 36] = [
                        00, 01, 02, 03, 24, 25,
                        04, 05, 06, 07, 26, 27,
                        08, 09, 10, 11, 28, 29,
                        12, 13, 14, 15, 30, 31,
                        16, 17, 18, 19, 32, 33,
                        20, 21, 22, 23, 34, 35,
                    ];

                    for (section_index, section) in frame.sections.iter().enumerate() {
                        sorted_sections[SECTION_MAPPING[section_index]] = *section;
                    }

                    bytes.extend(sorted_sections.iter().flat_map(
                        |section| section.bytes.iter().flat_map(
                            |row| row.iter().map(|&b| b)
                        )
                    ));
                }
            }

            self.file.lock().unwrap().seek(SeekFrom::Start(character.sprite_offset as u64))?;
            let mut byte_folder = ByteFolder::new(bytes.into_iter());
            self.file.lock().unwrap().write(byte_folder.collect::<Vec<_>>().as_slice())?;
        }
        Ok(())
    }

    pub fn save_spritesheet(&self, palette_manager: &mut palette::PaletteManager, character: &Character) -> Result<(), Error> {
        let spritesheet_o = self.spritesheets.get(&character.name.to_string());
        if let Some(spritesheet) = spritesheet_o {
            let palette = palette_manager.load_palette_colors(character.name.to_string());
            spritesheet.to_img(&palette[..]).save(format!("roms/sprites/{}.png", character.name))?;
        }
        Ok(())
    }

    pub fn load_spritesheet(&self, character: &Character) -> Result<&Spritesheet, Error> {
        let result = self.spritesheets.get(&character.name.to_string());
        match result {
            Some(spritesheet) => return Ok(spritesheet),
            None => return Err(Error::new(ErrorKind::InvalidData, format!("invalid character {}", &character.name.to_string())))
        }
    }
}

struct ByteFolder<I: Iterator<Item=u8>> {
    inner: I
}

impl<I: Iterator<Item=u8>> ByteFolder<I> {
    fn new(iter: I) -> Self {
        ByteFolder {
            inner: iter
        }
    }
}

impl<I: Iterator<Item=u8>> Iterator for ByteFolder<I> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let first = self.inner.next()?;
        let second = self.inner.next()?;
        Some(first | (second << 4))
    }
}
