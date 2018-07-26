#[macro_use]
extern crate lazy_static;
extern crate gtk;
extern crate image;

use gtk::prelude::*;
use gtk::*;

use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Read;
use std::error::Error;
use std::env;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::time::Instant;
use self::image::{open, ImageBuffer, Rgb, DynamicImage, ImageRgb8, ImageRgba8, ConvertBuffer};

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
    engine.lock().unwrap().start().unwrap();
    start_gui();
}

// make moving clones into closures more convenient
// stolen from gtk examples
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

fn start_gui() {
    gtk::init().unwrap();

    let glade_src = include_str!("sbrx_gui.glade");
    let builder: Rc<Builder> = Rc::new(Builder::new());
    builder.add_from_string(glade_src).expect("Couldn't add from string");

    let window: gtk::ApplicationWindow = builder.get_object("application").expect("Couldn't get window");
    window.set_title(&format!("sbrx v{}", VERSION));

    let character_options: Rc<gtk::ComboBoxText> = Rc::new(builder.get_object("character_options").expect("Couldn't get builder"));

    let character_upload_button: gtk::Button = builder.get_object("character_upload_file").expect("Couldn't get builder");
    let character_save_button: gtk::Button = builder.get_object("character_save_to_file").expect("Couldn't get builder");
    let character_write_button: gtk::Button = builder.get_object("character_write_to_rom").expect("Couldn't get builder");

    character_options.remove_all();

    for character in data::CHARACTERS.iter() {
        character_options.append(Some(character.name), character.name);
    }

    // Change character
    character_options.connect_changed(clone!(builder, character_options => move |_| {
        if let Some(text) = character_options.get_active_text() {
            let mut e = ENGINE.clone();
            let mut engine = e.lock().unwrap();
            let character = data::CHARACTERS.iter().filter(|&c| c.name == text).nth(0).unwrap();
            println!("Switching to character: {}", character.name);

            let palette = engine.palette_manager.load_palette_colors(character.name.to_string());
            let image = { engine.sprite_manager.load_spritesheet(character).unwrap().to_img(&palette[..]) };

            create_dir_all("/tmp/sbrx/").unwrap();
            let file_name = format!("/tmp/sbrx/{}.png", character.name);
            image.save(file_name.clone()).unwrap();

            let character_spritesheet: gtk::Image = builder.get_object("character_spritesheet")
                .expect("Couldn't get builder");

            character_spritesheet.set_from_file(file_name.clone());
        }
    }));

    // Upload spritesheet
    character_upload_button.connect_clicked(clone!(builder, character_options => move |_| {
        if let Some(text) = character_options.get_active_text() {
            let mut e = ENGINE.clone();
            let mut engine = e.lock().unwrap();
            let character = data::CHARACTERS.iter().filter(|&c| c.name == text).nth(0).unwrap();

            let window: gtk::ApplicationWindow = builder.get_object("application").expect("Couldn't get window");

            let total_timer = Instant::now();

            let dialog = FileChooserDialog::new(
                Some(&format!("Choose a spritesheet to open for {}", character.name)),
                Some(&window),
                FileChooserAction::Open
            );

            dialog.add_button("Cancel", 0);
            dialog.add_button("Open", 1);
            dialog.run();

            dialog.connect_response(clone!(builder, dialog => move |_, response_id| {
                match response_id {
                    0 => {
                        println!("Cancelling {}", character.name);
                        dialog.emit_close();
                    },
                    1 => {
                        println!("Opening {}", character.name);

                        let mut e = ENGINE.clone();
                        let mut engine = e.lock().unwrap();

                        let file_name = dialog.get_filename().unwrap();
                        let dynamic_image = open(file_name.clone()).ok().expect("Couldn't open image");
                        let mut image = match dynamic_image {
                            ImageRgb8(mut image) => {
                                image
                            }
                            ImageRgba8(mut image) => {
                                let converted_image: ImageBuffer<Rgb<u8>, Vec<u8>> = image.convert();
                                converted_image
                            }
                            _ => {
                                panic!("Couldn't open image {:?}", file_name.clone());
                            }
                        };

                        let (spritesheet, palette) = manager::sprite::Spritesheet::from_img(&mut image, character).unwrap();
                        engine.sprite_manager.spritesheets.insert(character.name.to_string(), spritesheet);
                        engine.palette_manager.store_palette_colors(character.name.to_string(), palette);
                        println!("Converted & stored spritesheet");

                        create_dir_all("/tmp/sbrx/").unwrap();
                        let tmp_file_name = format!("/tmp/sbrx/{}_upload_tmp.png", character.name);
                        image.save(tmp_file_name.clone()).unwrap();

                        let character_spritesheet: gtk::Image = builder.get_object("character_spritesheet")
                            .expect("Couldn't get builder");

                        character_spritesheet.set_from_file(tmp_file_name.clone());

                        dialog.emit_close();
                    },
                    _ => {}
                }
            }));

            println!("Upload image to character: {} ({:?})", character.name, total_timer.elapsed());
        }
    }));

    // Save to file
    character_save_button.connect_clicked(clone!(builder, character_options, window => move |_| {
        if let Some(text) = character_options.get_active_text() {
            let mut e = ENGINE.clone();
            let mut engine = e.lock().unwrap();
            let character = data::CHARACTERS.iter().filter(|&c| c.name == text).nth(0).unwrap();

            let window: gtk::ApplicationWindow = builder.get_object("application").expect("Couldn't get window");

            let total_timer = Instant::now();

            let dialog = FileChooserDialog::new(
                Some(&format!("Choose where to save the {} spritesheet", character.name)),
                Some(&window),
                FileChooserAction::Save
            );

            dialog.add_button("Cancel", 0);
            dialog.add_button("Save", 1);
            dialog.run();

            dialog.connect_response(clone!(builder, dialog => move |_, response_id| {
                match response_id {
                    0 => {
                        println!("Cancelling {}", character.name);
                        dialog.emit_close();
                    },
                    1 => {
                        println!("Saving {}", character.name);

                        let mut e = ENGINE.clone();
                        let mut engine = e.lock().unwrap();

                        let file_name = dialog.get_filename().unwrap();

                        let palette = engine.palette_manager.load_palette_colors(character.name.to_string());
                        let image = { engine.sprite_manager.load_spritesheet(character).unwrap().to_img(&palette[..]) };

                        image.save(file_name).unwrap();

                        dialog.emit_close();
                    },
                    _ => {}
                }
            }));

            println!("Save character to file: {} ({:?})", character.name, total_timer.elapsed());
        }
    }));

    // Write to ROM
    character_write_button.connect_clicked(clone!(builder, character_options => move |_| {
        if let Some(text) = character_options.get_active_text() {
            let mut e = ENGINE.clone();
            let mut engine = e.lock().unwrap();
            let character = data::CHARACTERS.iter().filter(|&c| c.name == text).nth(0).unwrap();

            let total_timer = Instant::now();
            let mut timer = Instant::now();
            engine.palette_manager.write_palette(character).unwrap();
            engine.sprite_manager.write_spritesheet(character).unwrap();
            println!("Write character to rom: {} ({:?})", character.name, timer.elapsed());

            timer = Instant::now();
            engine.palette_manager.read_palette(character).unwrap();
            engine.sprite_manager.read_sprite(character).unwrap();
            println!("Reading {} sprites & palette from ROM ({:?})", character.name, timer.elapsed());

            timer = Instant::now();
            let palette = engine.palette_manager.load_palette_colors(character.name.to_string());
            let image = { engine.sprite_manager.load_spritesheet(character).unwrap().to_img(&palette[..]) };
            println!("Converting {} spritesheet to an image ({:?})", character.name, timer.elapsed());

            timer = Instant::now();
            create_dir_all("/tmp/sbrx/").unwrap();
            let file_name = format!("/tmp/sbrx/{}.png", character.name);
            image.save(file_name.clone()).unwrap();

            let character_spritesheet: gtk::Image = builder.get_object("character_spritesheet")
                .expect("Couldn't get builder");

            character_spritesheet.set_from_file(file_name.clone());
            println!("Updating {} spritesheet gui ({:?})", character.name, timer.elapsed());

            println!("{} Write to ROM done ({:?})", character.name, total_timer.elapsed());
        }
    }));

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    gtk::main();
}
