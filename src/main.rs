#[macro_use]
extern crate lazy_static;
extern crate gtk;

use gtk::prelude::*;
use gtk::*;

use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Read;
use std::error::Error;
use std::env;
use std::sync::{Arc, Mutex};

mod data;
mod color;
mod engine;
mod manager;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

lazy_static! {
    static ref ENGINE: Arc<Mutex<engine::Engine>> = {
        if env::args().len() > 1 {
            let file_name = env::args().nth(1).unwrap();
            let file_result = OpenOptions::new()
                .read(true)
                .write(true)
                .open(file_name);
            match file_result {
                Ok(file) => {
                    Arc::new(Mutex::new(engine::Engine::new(Arc::new(Mutex::new(file)))))
                }
                Err(error) => panic!("Error occurred while opening file: {}", error)
            }
        } else {
            panic!("No file specified!");
        }
    };
}

fn main() {
    let engine = ENGINE.clone();
    engine.lock().unwrap().start();
    start_gui();
}

fn start_gui() {
    gtk::init().unwrap();

    let glade_src = include_str!("sbrx_gui.glade");
    let builder = Builder::new();
    builder.add_from_string(glade_src).expect("Couldn't add from string");

    let window: gtk::ApplicationWindow = builder.get_object("application").expect("Couldn't get window");
    window.set_title(&format!("sbrx v{}", VERSION));

    let character_options: gtk::ComboBoxText = builder.get_object("character_options")
        .expect("Couldn't get builder");

    character_options.remove_all();

    for character in data::CHARACTERS.iter() {
        character_options.append(Some(character.name), character.name);
    }

    character_options.connect_changed(move |_| {
        let character_options_copy: gtk::ComboBoxText = builder.get_object("character_options")
            .expect("Couldn't get builder");
        if let Some(text) = character_options_copy.get_active_text() {
            let mut e = ENGINE.clone();
            let mut engine = e.lock().unwrap();
            let character = data::CHARACTERS.iter().filter(|&c| c.name == text).nth(0).unwrap();
            println!("Found character: {}", character.name);

            let palette = engine.palette_manager.load_palette_colors(character.name.to_string());
            let image = { engine.sprite_manager.load_spritesheet(character).unwrap().to_img(&palette[..]) };

            create_dir_all("/tmp/sbrx/").unwrap();
            let file_name = format!("/tmp/sbrx/{}.png", character.name);
            image.save(file_name.clone()).unwrap();

            let character_spritesheet: gtk::Image = builder.get_object("character_spritesheet")
                .expect("Couldn't get builder");

            character_spritesheet.set_from_file(file_name.clone());
        }
    });

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    gtk::main();
}
