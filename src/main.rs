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
use std::{env, thread};

mod conrod_support;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
fn main() {
    let mut args = env::args();
    args.nth(0);
    #[cfg(not(target_os = "windows"))]
    let path_string = env::var("HOME").expect("Failed to get your home directory") + "/.minecraft";
    #[cfg(target_os = "windows")]
    let path_string = env::var("APPDATA").expect("Failed to get APPDATA directory") + "/.minecraft";
    let url_string = "https://github.com/ssf-tf/mc-pack1.git";
    let path = format!("{0}/.tmp_git/{1}", path_string, base64::encode(url_string));
    use git2::Repository;

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
    let url = url_string.clone();
    let path_git = path.clone();
    let thread_git = thread::spawn(move || {
        let repo = match Repository::open(&path_git) {
            Ok(repo) => repo,
            Err(_) => Repository::clone(url, path_git).unwrap(),
        };
        let mut remote = match repo.find_remote("origin") {
            Ok(r) => r,
            Err(_) => repo.remote("origin", url).unwrap(),
        };
        remote.download(&[""], None).unwrap();
        remote.fetch(&["master"], None, None).unwrap();
        let oid = repo.refname_to_id("refs/remotes/origin/master").unwrap();
        let object = repo.find_object(oid, None).unwrap();
        repo.reset(&object, git2::ResetType::Hard, None).unwrap();

        send.send(true).unwrap();
    });
    let mut text = "Installing";
    let mut green = 0.14;
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

        if let Ok(_) = recv.try_recv() {
            text = "DONE";
            green = 0.6;
            if cfg!(target_os = "windows") {
                use std::process::Command;
                fn robcop(from: &str, to: &str, dir: &str, mir: bool) {
                    Command::new("cmd")
                        .args(&[
                            "/C",
                            &format!(
                                "robocopy {1}/{2} {0}/{2} /{3}",
                                to,
                                from,
                                dir,
                                if mir { "mir" } else { "s" }
                            ),
                        ])
                        .output()
                        .expect("failed to robocopy");
                }
                robcop(&path_string, &path, "mods", true);
                robcop(&path_string, &path, "config", true);
                robcop(&path_string, &path, "scripts", true);

                robcop(&path_string, &path, "libraries", false);
                robcop(&path_string, &path, "resources", false);
                robcop(&path_string, &path, "versions", false);
            } else {
                //TODO linux folder copy
            };
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
widget_ids!(struct Ids { can,t1,b1 });
