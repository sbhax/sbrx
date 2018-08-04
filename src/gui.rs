extern crate nfd;

use conrod;
use self::nfd::Response;
use self::engine::*;
use std::sync::{Arc, Mutex};
use image::open;
use glium;

use self::super::*;
use self::super::data::*;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub struct GuiState {
    chosen_file: String,
    selected_character_index: Option<usize>,
    engine: Option<Engine>,
    spritesheet: Option<conrod::image::Id>,
}

impl GuiState {
    pub fn new(engine: Option<Engine>) -> Self {
        GuiState {
            engine,
            selected_character_index: None,
            chosen_file: "no ROM open".to_string(),
            spritesheet: None,
        }
    }

    pub fn get_character(&self) -> Option<Character> {
        if let Some(index) = self.selected_character_index {
            Some(CHARACTERS[index])
        } else {
            None
        }
    }

    pub fn insert_image(&mut self, display: &glium::Display, image_map: &mut conrod::image::Map<glium::texture::Texture2d>, image: ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let rgba = image::ImageRgb8(image).to_rgba();
        let dimensions = rgba.dimensions();
        let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&rgba.into_raw(), dimensions);
        let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
        self.spritesheet = Some(image_map.insert(texture))
    }
}

pub fn theme() -> conrod::Theme {
    use conrod::position::{Align, Direction, Padding, Position, Relative};
    conrod::Theme {
        name: "sbrx theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod::color::DARK_CHARCOAL,
        shape_color: conrod::color::LIGHT_RED,
        border_color: conrod::color::BLACK,
        border_width: 0.0,
        label_color: conrod::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}

// create the ids
widget_ids! {
    pub struct Ids {
        canvas,
        canvas_scrollbar,

        title,
        subtitle,

        file_chooser_button,
        file_chooser_text,

        character_dropdown,
        spritesheet_upload,
        spritesheet_save,
        spritesheet_write,
        spritesheet,
    }
}

pub fn gui(display: &glium::Display, image_map: &mut conrod::image::Map<glium::texture::Texture2d>, ui: &mut conrod::UiCell, ids: &Ids, app: &mut GuiState) {
    use conrod::{widget, Colorable, Labelable, Positionable, Sizeable, Widget};
    use std::iter::once;

    const MARGIN: conrod::Scalar = 30.0;
    const SHAPE_GAP: conrod::Scalar = 50.0;
    const TITLE_SIZE: conrod::FontSize = 42;
    const SUBTITLE_SIZE: conrod::FontSize = 32;

    widget::Canvas::new().pad(MARGIN).scroll_kids_vertically().set(ids.canvas, ui);

    widget::Text::new("sbrx")
        .font_size(TITLE_SIZE)
        .top_left_of(ids.canvas)
        .set(ids.title, ui);

    widget::Text::new(&format!("version {} by phase", VERSION))
        .padded_w_of(ids.canvas, MARGIN)
        .top_right_of(ids.canvas)
        .down(5.0)
        .line_spacing(5.0)
        .set(ids.subtitle, ui);

    for _press in widget::Button::new()
        .label("Open ROM")
        .small_font(ui)
        .top_right_of(ids.canvas)
        .w_h(70.0, 25.0)
        .set(ids.file_chooser_button, ui)
        {
            let result = nfd::dialog().filter("gba").open().unwrap_or_else(|e| {
                panic!(e);
            });
            match result {
                Response::Okay(file_name) => {
                    println!("File path = {:?}", file_name);
                    app.chosen_file = file_name.clone();
                    let file_result = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(file_name);
                    match file_result {
                        Ok(file) => {
                            let mut engine = engine::Engine::new(Arc::new(Mutex::new(file)));
                            if let Ok(_) = engine.start() {
                                app.engine = Some(engine);
                                app.selected_character_index = None;
                            }
                        }
                        Err(error) => {
                            println!("Error occurred while opening file: {}", error);
                        }
                    }
                }
                Response::Cancel => println!("User canceled"),
                _ => (),
            }
        }

    widget::Text::new(&app.chosen_file)
        .bottom_right_of(ids.file_chooser_button)
        .font_size(10)
        .down(5.0)
        .align_right()
        .set(ids.file_chooser_text, ui);

    //
    // Spritesheets
    //

    for selected_index in widget::DropDownList::new(CHARACTERS.iter().map(|c| c.name).collect::<Vec<&str>>().as_slice(), app.selected_character_index)
        .small_font(ui)
        .bottom_left_of(ids.subtitle)
        .down(50.0)
        .w_h(60.0, 25.0)
        .set(ids.character_dropdown, ui)
        {
            // Change character
            app.selected_character_index = Some(selected_index);

            let o_image = (if let Some(ref mut engine) = app.engine {
                println!("Loading character data");
                let character = CHARACTERS[selected_index];
                let palette = engine.palette_manager.load_palette_colors(character.name.to_string());
                Some(engine.sprite_manager.load_spritesheet(&character).unwrap().to_img(&palette[..]))
            } else {
                None
            });

            if let Some(image) = o_image {
                app.insert_image(display, image_map, image);
            }
        }

    for _press in widget::Button::new()
        .label("Upload Spritesheet")
        .small_font(ui)
        .bottom_left_of(ids.subtitle)
        .down(20.0)
        .w_h(115.0, 25.0)
        .set(ids.spritesheet_upload, ui)
        {
            println!("Upload Spritesheet");
            if let Some(character) = app.get_character() {
                if let Some(ref mut engine) = app.engine {
                    let result = nfd::dialog().filter("png").open().unwrap_or_else(|e| {
                        panic!(e);
                    });
                    match result {
                        Response::Okay(file_name) => {
                            println!("File path = {:?}", file_name);
                            app.chosen_file = file_name.clone();
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

                            let (spritesheet, palette) = manager::sprite::Spritesheet::from_img(&mut image, &character).unwrap();
                            engine.sprite_manager.spritesheets.insert(character.name.to_string(), spritesheet);
                            engine.palette_manager.store_palette_colors(character.name.to_string(), palette);
                            println!("Converted & stored spritesheet");
                        }
                        Response::Cancel => println!("User canceled"),
                        _ => (),
                    }
                }
            }
        }

    for _press in widget::Button::new()
        .label("Save Spritesheet to File")
        .small_font(ui)
        .down(20.0)
        .w_h(140.0, 25.0)
        .set(ids.spritesheet_save, ui)
        {
            println!("Save Spritesheet to File");
            if let Some(character) = app.get_character() {
                if let Some(ref mut engine) = app.engine {
                    let palette = engine.palette_manager.load_palette_colors(character.name.to_string());
                    let image = { engine.sprite_manager.load_spritesheet(&character).unwrap().to_img(&palette[..]) };

                    let result = nfd::dialog_save().filter("png").open().unwrap_or_else(|e| {
                        panic!(e);
                    });

                    match result {
                        Response::Okay(file_name) => {
                            let file = if file_name.ends_with(".png") {
                                file_name
                            } else {
                                format!("{}.png", file_name)
                            };
                            image.save(file).unwrap();
                        }
                        Response::Cancel => println!("User canceled"),
                        _ => (),
                    }
                }
            }
        }

    for _press in widget::Button::new()
        .label("Write Spritesheet to ROM")
        .small_font(ui)
        .down(20.0)
        .w_h(150.0, 25.0)
        .set(ids.spritesheet_write, ui)
        {
            println!("Write Spritesheet to ROM");
            if let Some(character) = app.get_character() {
                if let Some(image) = if let Some(ref mut engine) = app.engine {
                    let total_timer = Instant::now();
                    let mut timer = Instant::now();
                    engine.palette_manager.write_palette(&character).unwrap();
                    engine.sprite_manager.write_spritesheet(&character).unwrap();
                    println!("Write character to rom: {} ({:?})", character.name, timer.elapsed());

                    timer = Instant::now();
                    engine.palette_manager.read_palette(&character).unwrap();
                    engine.sprite_manager.read_sprite(&character).unwrap();
                    println!("Reading {} sprites & palette from ROM ({:?})", character.name, timer.elapsed());

                    timer = Instant::now();
                    let palette = engine.palette_manager.load_palette_colors(character.name.to_string());
                    let image = { engine.sprite_manager.load_spritesheet(&character).unwrap().to_img(&palette[..]) };
                    println!("Converting {} spritesheet to an image ({:?})", character.name, timer.elapsed());
                    Some(image)
                } else { None } {
                    // update display
                    app.insert_image(display, image_map, image);
                }
            }
        }

    if let Some(ref image) = app.spritesheet {
        widget::Image::new(*image)
//            .w_h(400.0,60.0)
            .middle()
            .w_h(800.0, 400.0)
            .down(40.0)
            .set(ids.spritesheet, ui);
    }

    widget::Scrollbar::y_axis(ids.canvas).auto_hide(true).set(ids.canvas_scrollbar, ui);
}

pub struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

// taken from the conrod examples
impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(&mut self, events_loop: &mut glium::glutin::EventsLoop) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events_loop.poll_events(|event| events.push(event));

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events_loop.run_forever(|event| {
                events.push(event);
                glium::glutin::ControlFlow::Break
            });
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
}
