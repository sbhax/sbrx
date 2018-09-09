#![windows_subsystem = "windows"]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate conrod;
extern crate image;
extern crate discord_rpc_client;

use std::io;
use discord_rpc_client::Client as DiscordRPC;

use conrod::backend::glium::glium;
use conrod::backend::glium::glium::Surface;
use conrod::text::Font;

use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Read;
use std::error::Error;
use std::env;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::time::Instant;
use self::image::{open, ImageBuffer, Rgb, DynamicImage, ImageRgb8, ImageRgba8, ConvertBuffer};

mod gui;
mod data;
mod color;
mod engine;
mod manager;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const WINDOW_WIDTH: u32 = gui::WINDOW_WIDTH;
const WINDOW_HEIGHT: u32 = gui::WINDOW_HEIGHT;

pub fn main() {

	// Discord work by @LavenderTGreat
	// This is my first time using rust to it may be terrible.
	
    let mut drpc = DiscordRPC::new(488064542417485844)
		.expect("Failed to create Discord Rich Presence client");
	
	drpc.start();
	
	if let Err(why) = drpc.set_activity(|a| a
		.assets(|ass| ass
			.large_image("mainnew")
			.large_text("SBRX by Phase & co."))
		.details("Hacking Sonic Battle"))
	{
		println!("Failed to set Rich Presence: {}", why);
	}
	
	// Discord work: End
	
	let mut events_loop = glium::glutin::EventsLoop::new();

    let window = glium::glutin::WindowBuilder::new()
        .with_title(format!("sbrx v{}", VERSION))
        .with_dimensions((WINDOW_WIDTH, WINDOW_HEIGHT).into());

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    let mut image_map: conrod::image::Map<glium::texture::Texture2d> = conrod::image::Map::new();

    let mut ui = conrod::UiBuilder::new([WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64]).theme(gui::theme()).build();

    let font = Font::from_bytes(include_bytes!("assets/NotoSans-Regular.ttf").to_vec()).unwrap();
    ui.fonts.insert(font);

    let ids = gui::Ids::new(ui.widget_id_generator());

    let engine: Option<engine::Engine> = if env::args().len() > 1 {
        let file_name = env::args().nth(1).unwrap();
        let file_result = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_name);
        match file_result {
            Ok(file) => {
                let mut engine = engine::Engine::new(Arc::new(Mutex::new(file)));
                if let Ok(_) = engine.start() {
                    Some(engine)
                } else {
                    None
                }
            }
            Err(error) => {
                println!("Error occurred while opening file: {}", error);
                None
            }
        }
    } else {
        println!("No file specified!");
        None
    };

    let mut app = gui::GuiState::new(engine);
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    let mut event_loop = gui::EventLoop::new();
    'main: loop {

        for event in event_loop.next(&mut events_loop) {

            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    glium::glutin::WindowEvent::CloseRequested |
                    glium::glutin::WindowEvent::KeyboardInput {
                        input: glium::glutin::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        gui::gui(&display, &mut image_map, &mut ui.set_widgets(), &ids, &mut app, &mut drpc); // Added &mut drpc to the end as you need to pass it through for the change on character select

        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}

