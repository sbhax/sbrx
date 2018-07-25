use std::fs::{File, OpenOptions};
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
        let file_result = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_name);
        match file_result {
            Ok(mut file) => {
                let mut engine = engine::Engine::new(&mut file);
                match engine.start() {
                    Ok(_) => {}
                    Err(error) => panic!("Error occured: {}", error.description())
                }
            }
            Err(error) => panic!("Error occurred while opening file: {}", error)
        }
    } else {
        println!("No file specified!")
    }
}
