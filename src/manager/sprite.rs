use std::collections::HashMap;
use std::fs::File;
use std::io::*;

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
}
