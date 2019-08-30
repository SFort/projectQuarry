#[macro_use]
extern crate conrod_core;
#[macro_use]
extern crate conrod_winit;
#[macro_use]
extern crate rust_embed;
extern crate base64;
extern crate conrod_glium;
extern crate git2;
extern crate glium;
#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;
use conrod_core::{text, Colorable};
use glium::Surface;
use std::{env, thread, time::Instant};

mod conrod_support;
mod sfl;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
fn main() {
    let mut args = env::args();
    args.nth(0);
    #[cfg(not(target_os = "windows"))]
    let path_root = env::var("HOME").expect("Failed to get your home directory") + "/.minecraft";
    #[cfg(target_os = "windows")]
    let path_root = env::var("APPDATA").expect("Failed to get APPDATA directory") + "/.minecraft";
    let url = if let Some(input) = args.nth(0) {
        input
    } else {
        "https://github.com/ssf-tf/mc-pack1.git".into()
    };
    let path = format!("{0}/.tmp_git/{1}", path_root, base64::encode(&url));

    const WIDTH: u32 = 480;
    const HEIGHT: u32 = 480;

    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("SFs")
        .with_dimensions((WIDTH, HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = conrod_support::GliumDisplayWinitWrapper(display);
    let ids = Ids::new(ui.widget_id_generator());

    let font = text::Font::from_bytes(Asset::get("serif.ttf").unwrap().to_vec()).unwrap();
    ui.fonts.insert(font);
    let mut renderer = conrod_glium::Renderer::new(&display.0).unwrap();
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();
    let mut event_loop = conrod_support::EventLoop::new();
    let (send, recv) = std::sync::mpsc::channel();
    //off thread the install
    let path_lib = path_root.clone();
    let path_to = if let Some(input) = args.nth(0) {
        input
    } else {
        path_root.into()
    };
    let thread_git = thread::spawn(move || {
        sfl::update(&path, &url, &send);

        send.send(Some("Moving Files".to_string())).unwrap();
        if cfg!(target_os = "windows") {
            use sfl::robcop;
            robcop(&path, &path_to, "mods", true);
            robcop(&path, &path_to, "config", true);
            robcop(&path, &path_to, "scripts", true);
            robcop(&path, &path_to, "resources", false);
            robcop(&path, &path_lib, "versions", false);
        } else {
            //TODO linux folder copy
        };
        send.send(Some("Moved Files".to_string())).unwrap();
        send.send(None).unwrap();
    });
    let mut text = "Installing";
    let mut text_log = String::new();
    let mut green = 0.14;
    let mut autokill: Option<Instant> = None;
    'render: loop {
        for event in event_loop.next(&mut events_loop) {
            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod_support::convert_event(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    glium::glutin::WindowEvent::CloseRequested => break 'render,
                    _ => (),
                },
                _ => (),
            }
        }
        let ui = &mut ui.set_widgets();
        use conrod_core::widget::{Canvas, Text};
        use conrod_core::{color::hsl, color::rgb, Positionable, Widget};

        if let Some(time) = autokill {
            if time.elapsed().as_secs() > 4 {
                break 'render;
            }
        }

        if let Ok(x) = recv.try_recv() {
            if let Some(message) = x {
                text_log = message;
            } else {
                autokill = Some(Instant::now());
                text = "DONE";
                green = 0.6;
            }
        }
        Canvas::new()
            .pad(15.0)
            .color(rgb(0.14, green, 0.14))
            .set(ids.can, ui);
        Text::new(&format!("SForts  Installer  V.: {}", CURRENT_VERSION))
            .mid_top_of(ids.can)
            .color(hsl(1.0, 1.0, 1.0))
            .font_size(32)
            .set(ids.t1, ui);
        Text::new(text)
            .middle_of(ids.can)
            .color(hsl(1.0, 1.0, 1.0))
            .font_size(48)
            .set(ids.b1, ui);
        Text::new(&text_log)
            .mid_bottom_of(ids.can)
            .color(hsl(1.0, 1.0, 1.0))
            .font_size(16)
            .set(ids.l1, ui);
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display.0, primitives, &image_map);
            let mut target = display.0.draw();
            target.clear_color(0.024, 0.024, 0.024, 1.0);
            renderer.draw(&display.0, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    } //End render loop

    thread_git.join().unwrap();
}
widget_ids!(struct Ids { can,t1,b1,l1 });
