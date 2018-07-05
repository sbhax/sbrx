use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::env;

mod data;
mod color;
mod engine;
mod manager;

fn main() {
    if env::args().len() > 1 {
        let file_name = env::args().nth(1).unwrap();
        let file_result = File::open(file_name);
        match file_result {
            Ok(file) => {
                let mut engine = engine::Engine::new(file);
                engine.start();
            }
            Err(error) => panic!("Error occurred while opening file: {}", error.description())
        }
    } else {
        println!("No file specified!")
    }
    println!("Sonic Palette Offset: {}", data::SONIC_DATA.palette_offset);
}
