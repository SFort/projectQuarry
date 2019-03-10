#[cfg(not(target_os = "windows"))]
extern crate cursive;
extern crate ini;
use ini::Ini;
use std::io::{self, Write};
use std::path::Path;
use std::{env, fs};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
fn main() {
    let mut args = env::args();
    args.nth(0);
    let arg1: &str = &args.nth(0).unwrap_or("ui".into()).to_lowercase();
    match arg1 {
        "ui" => loop {
            let get_input = |input_title: &str| -> String {
                let mut command = String::new();
                print!("{}", input_title);
                io::stdout().flush().unwrap();
                match io::stdin().read_line(&mut command) {
                    Ok(0) => command = "exit".into(),
                    Ok(_) => {}
                    Err(e) => panic!(e),
                }
                command.trim().to_string()
            };
            let backup_val = |bk_val: &str, input: String| -> String {
                if input.is_empty() {
                    return bk_val.to_string();
                }
                input
            };

            let mut command: &str = &get_input("ER> ");
            let mut args_str: Vec<String> = Vec::new();
            match command {
                //NOTE add new UI commands here
                "add" => args_str.push(format!(
                    "{}#{}",
                    get_input("Pack Link:  "),
                    get_input("Pack label(optional): ")
                )),
                "install" | "remove" => args_str.push(get_input("Specify pack: ")),
                "reload" => args_str.push(backup_val(
                    "all",
                    get_input("Specify pack   [ALL,pack_name ]:  "),
                )),
                "list" => command = "list".into(),
                "exit" | "quit" => break,
                _ => {
                    println!("Unknown command \n Available: add, list, install, reload, remove");
                    continue;
                }
            }
            println!("{}", run_command(command.into(), args_str));
        }, //NOTE end of UI         #############################################

        #[cfg(not(target_os = "windows"))]
        "tui" => {
            use cursive::traits::*;
            use cursive::views::{
                Button, Dialog, DummyView, EditView, LinearLayout, SelectView, TextView, ViewRef,
            };
            use cursive::Cursive;
            let mut tui = Cursive::default();
            tui.add_global_callback('q', |s| s.quit());
            let mut packs = SelectView::<String>::new()
                .on_submit(|s, pak: &String| {
                    s.call_on_id("return_info", |v: &mut TextView| {
                        v.set_content(format!(
                            "{}",
                            run_command("install".into(), vec![pak.clone()])
                        ))
                    });
                })
                .with_id("packs")
                .min_width(15);
            let mut left_layout = LinearLayout::vertical()
                .child(Button::new("Sync packs", |s| {
                    s.call_on_id("return_info", |v: &mut TextView| {
                        v.set_content(format!(
                            "{}",
                            run_command("reload".into(), vec!["all".into()])
                        ))
                    });
                }))
                .child(Button::new("Re-list packs", |s| {
                    tui_update_packs(s);
                }))
                .child(DummyView)
                .child(packs);

            let mut commands = LinearLayout::vertical()
                .child(DummyView)
                .child(DummyView)
                .child(Button::new("Add Pack", |s| {
                    tui_get_args(s, "add", vec!["Pack Link", "Pack Label(optional)"]);
                }))
                .child(
                    Button::new("Reload pack", |s| {
                        tui_get_args(s, "reload", vec!["Pack name"]);
                    })
                    .disabled()
                    .with_id("b_reload"),
                )
                .child(
                    Button::new("Remove pack", |s| {
                        tui_get_args(s, "remove", vec!["Pack name"]);
                    })
                    .disabled()
                    .with_id("b_remove"),
                );

            tui.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(
                            LinearLayout::horizontal()
                                .child(left_layout)
                                .child(DummyView)
                                .child(commands),
                        )
                        .child(TextView::new("").with_id("return_info")),
                )
                .title(format!("𝐸𝑅 Client   {}", CURRENT_VERSION)),
            );

            match run_command("get_tui_toml".into(), Vec::new()).rez {
                Ok(t) => tui.load_toml(&t).unwrap_or({}),
                Err(_) => {}
            }

            tui_update_packs(&mut tui);
            tui.run();

            fn tui_update_packs(s: &mut Cursive) {
                fn set_button_state(s: &mut Cursive, set_state: bool) {
                    for button in vec!["b_remove", "b_reload"] {
                        s.call_on_id(button, |v: &mut Button| v.set_enabled(set_state));
                    }
                };
                let mut packs_menu: ViewRef<SelectView> = s.find_id("packs").unwrap();
                packs_menu.clear();
                match run_command("list".into(), Vec::new()).rez {
                    Ok(x) => {
                        set_button_state(s, true);
                        let packs_vec: Vec<&str> = x.split("\n").filter(|x| x != &"").collect();
                        for pack in packs_vec {
                            let f_pack = PackEntry::from(pack);
                            packs_menu.add_item(f_pack.label, f_pack.url);
                        }
                    }
                    Err(_) => {
                        s.call_on_id("return_info", |v: &mut TextView| {
                            v.set_content("No packs found")
                        });
                        set_button_state(s, false);
                    }
                }
            }
            fn tui_get_args<S>(s: &mut Cursive, command: S, required_fields: Vec<&str>)
            where
                S: Into<String>,
            {
                let mut menu =
                    LinearLayout::vertical().child(TextView::new(command).with_id("comm"));
                let mut index = 0;
                for i in required_fields {
                    menu = menu
                        .child(Dialog::text(format!("{}", i)))
                        .child(EditView::new().with_id(format!("arg{}", index)));
                    index += 1;
                }
                s.add_layer(
                    Dialog::around(menu)
                        .title("Required arguments")
                        .button("Ok", move |s| {
                            let mut args_str: Vec<String> = Vec::new();
                            for i in 0..index {
                                args_str.push(get_value(s, i));
                            }
                            let command = s
                                .call_on_id("comm", |v: &mut TextView| v.get_content())
                                .unwrap();
                            s.call_on_id("return_info", |v: &mut TextView| {
                                v.set_content(format!(
                                    "{}",
                                    run_command(command.source().into(), args_str)
                                ))
                            });

                            s.pop_layer();
                        })
                        .button("Cancel", |s| {
                            s.pop_layer();
                        }),
                );
                fn get_value(s: &mut Cursive, arg: u8) -> String {
                    s.call_on_id(&format!("arg{}", arg), |v: &mut EditView| v.get_content())
                        .unwrap()
                        .to_string()
                }
            }
        } //NOTE end of TUI             #################################################

        x => {
            let args_str: Vec<String> = args.map(|y| y).collect();
            println!("{:?}", run_command(x.into(), args_str))
        }
    }
    fn run_command(command: String, args: Vec<String>) -> CommandResult {
        let default_config = Ini::load_from_str("[pack]\nreload=1").unwrap();

        #[cfg(not(target_os = "windows"))]
        let path_string = env::var("HOME").expect("Failed to get your home directory") + "/.config";
        #[cfg(target_os = "windows")]
        let path_string = env::var("USERPROFILE").expect("Failed to get your user directory");
        let path = Path::new(&path_string).join(".elemental-realms");
        let p_config = path.join("client.ini");
        let p_packs = path.join("packs.list");
        let mut config = Ini::new();

        if !path.exists() {
            fs::create_dir(&path).unwrap();
        }
        if p_config.exists() {
            config = Ini::load_from_file(&p_config).unwrap();
        } else {
            config = default_config.clone();
        }
        if !p_packs.exists() {
            fs::write(&p_packs, "").unwrap();
        }

        return match command.to_lowercase().as_ref() {
            "list" => CommandResult {
                rez: match fs::read_to_string(&p_packs).unwrap().as_ref() {
                    "" => Err("".into()),
                    a => Ok(a.into()),
                },
            },
            //TODO
            "add" => {
                if args.is_empty() {
                    return CommandResult {
                        rez: Err("No argument".into()),
                    };
                }
                let str_pak = if args.len() > 1 {
                    format!("{}#{}", args[0], args[1])
                } else {
                    args[0].to_string()
                };
                let packs_str = fs::read_to_string(&p_packs).unwrap();
                CommandResult {
                    rez: match fs::write(&p_packs, format!("{}{}\n", packs_str, str_pak)) {
                        Ok(_) => Ok(format!("Added {}", str_pak)),
                        Err(e) => Err(e.to_string()),
                    },
                }
            }
            //https://docs.rs/cursive/0.10.0/cursive/theme/index.html#themes
            #[cfg(not(target_os = "windows"))]
            "get_tui_toml" => CommandResult {
                rez: match fs::read_to_string(path.join("tui.toml")) {
                    Ok(t) => Ok(t),
                    Err(_) => Err("".into()),
                },
            },
            _ => CommandResult {
                rez: Err(format!("{}, {:?}", command, args)),
            },
        };
        fn get_packs(path: &Path) -> Vec<PackEntry> {
            let packs_str = fs::read_to_string(path).unwrap();
            let packs_vec: Vec<&str> = packs_str.split("\n").filter(|x| x != &"").collect();
            let mut packs: Vec<PackEntry> = Vec::new();
            for pack in packs_vec {
                packs.push(PackEntry::from(pack));
            }
            packs
        }
    } //NOTE end of run_command
} //NOTE end of main
struct PackEntry {
    label: String,
    url: String,
}
impl From<&str> for PackEntry {
    fn from(input: &str) -> PackEntry {
        let (url0, label) = input.split_at(input.rfind('#').unwrap());
        let mut url = url0.to_string();
        url.pop();
        PackEntry {
            label: label.into(),
            url: url,
        }
    }
}
impl std::fmt::Debug for PackEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}#{}", self.url, self.label)
    }
}
struct CommandResult {
    rez: Result<String, String>,
}
impl std::fmt::Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.rez {
            Ok(s) => write!(f, "Success:\n  {}", s),
            Err(e) => write!(f, "Failed:\n  {}", e),
        }
    }
}
impl std::fmt::Debug for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.rez)
    }
}
