#![windows_subsystem = "windows"]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate conrod;
extern crate image;

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

lazy_static! {
    static ref ENGINE: Arc<Mutex<Option<engine::Engine>>> = {
        if env::args().len() > 1 {
            let file_name = env::args().nth(1).unwrap();
            let file_result = OpenOptions::new()
                .read(true)
                .write(true)
                .open(file_name);
            match file_result {
                Ok(file) => {
                    Arc::new(Mutex::new(Some(engine::Engine::new(Arc::new(Mutex::new(file))))))
                },
                Err(error) => {
                    println!("Error occurred while opening file: {}", error);
                    Arc::new(Mutex::new(None))
                }
            }
        } else {
            println!("No file specified!");
            Arc::new(Mutex::new(None))
        }
    };
}

pub fn main() {
    let mut events_loop = glium::glutin::EventsLoop::new();

    let window = glium::glutin::WindowBuilder::new()
        .with_title(format!("sbrx v{}", VERSION))
        .with_dimensions((WINDOW_WIDTH, WINDOW_HEIGHT).into());

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    let mut image_map = conrod::image::Map::new();

    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let (render_tx, render_rx) = std::sync::mpsc::channel();
    let events_loop_proxy = events_loop.create_proxy();

    fn run_conrod(
        event_rx: std::sync::mpsc::Receiver<conrod::event::Input>,
        render_tx: std::sync::mpsc::Sender<conrod::render::OwnedPrimitives>,
        events_loop_proxy: glium::glutin::EventsLoopProxy)
    {
        let mut ui = conrod::UiBuilder::new([WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64]).theme(gui::theme()).build();

        let font = Font::from_bytes(include_bytes!("assets/NotoSans-Regular.ttf").to_vec()).unwrap();
        ui.fonts.insert(font);


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

        let ids = gui::Ids::new(ui.widget_id_generator());

        let mut needs_update = true;
        'conrod: loop {
            let mut events = Vec::new();
            while let Ok(event) = event_rx.try_recv() {
                events.push(event);
            }

            if events.is_empty() || !needs_update {
                match event_rx.recv() {
                    Ok(event) => events.push(event),
                    Err(_) => break 'conrod,
                };
            }

            needs_update = false;

            for event in events {
                ui.handle_event(event);
                needs_update = true;
            }

            gui::gui(&mut ui.set_widgets(), &ids, &mut app);

            if let Some(primitives) = ui.draw_if_changed() {
                if render_tx.send(primitives.owned()).is_err()
                    || events_loop_proxy.wakeup().is_err() {
                    break 'conrod;
                }
            }
        }
    }

    fn draw(display: &glium::Display,
            renderer: &mut conrod::backend::glium::Renderer,
            image_map: &conrod::image::Map<glium::Texture2d>,
            primitives: &conrod::render::OwnedPrimitives)
    {
        renderer.fill(display, primitives.walk(), &image_map);
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        renderer.draw(display, &mut target, &image_map).unwrap();
        target.finish().unwrap();
    }

    std::thread::spawn(move || run_conrod(event_rx, render_tx, events_loop_proxy));

    let mut last_update = std::time::Instant::now();
    let mut closed = false;
    while !closed {
        // 60 fps
        let sixteen_ms = std::time::Duration::from_millis(16);
        let now = std::time::Instant::now();
        let duration_since_last_update = now.duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        events_loop.run_forever(|event| {
            // convert winit event to conrod event
            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                event_tx.send(event).unwrap();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested |
                    glium::glutin::WindowEvent::KeyboardInput {
                        input: glium::glutin::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => {
                        closed = true;
                        return glium::glutin::ControlFlow::Break;
                    }

                    glium::glutin::WindowEvent::Resized(..) => {
                        if let Some(primitives) = render_rx.iter().next() {
                            draw(&display, &mut renderer, &image_map, &primitives);
                        }
                    }
                    _ => {}
                },
                glium::glutin::Event::Awakened => return glium::glutin::ControlFlow::Break,
                _ => (),
            }

            glium::glutin::ControlFlow::Continue
        });

        if let Some(primitives) = render_rx.try_iter().last() {
            draw(&display, &mut renderer, &image_map, &primitives);
        }

        last_update = std::time::Instant::now();
    }
}

