extern crate gio;
extern crate gtk;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Mutex,RwLock};

use gio::prelude::*;
use gtk::prelude::*;
use gtk::traits::*;
use glib::clone;

use gtk::SearchEntryExt;
use gtk::{ApplicationWindow, Builder, Button, ListBox, Revealer, SearchEntry};

use std::env;
use std::env::args;
//use tokio_compat::prelude::*;
use tokio_core::reactor::Core;

extern crate clap;
extern crate crossbeam_channel;
#[macro_use]
#[cfg(feature = "share_clipboard")]
extern crate clipboard;
extern crate directories;
extern crate failure;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate librespot_core;
extern crate librespot_playback;
extern crate librespot_protocol;
extern crate rspotify;
extern crate tokio_core;
extern crate tokio_timer;
extern crate unicode_width;
extern crate webbrowser;

#[cfg(feature = "mpris")]
extern crate dbus;

#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate toml;

#[macro_use]
extern crate log;
extern crate chrono;
extern crate fern;

extern crate rand;
extern crate url;

extern crate strum;
extern crate strum_macros;

use std::fs;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::sync::Arc;

// use async_std::task;
use std::thread;

use clap::{App, Arg};

use librespot_core::authentication::Credentials;

mod album;
mod artist;
mod authentication;
mod command;
mod commands;
mod config;
mod events;
mod library;
mod playlist;
mod queue;
mod spotify;
mod track;
mod traits;
mod search;

#[cfg(feature = "mpris")]
mod mpris;

use crate::commands::CommandManager;
use crate::events::{Event, EventManager};
use crate::library::Library;
use crate::spotify::PlayerEvent;
use crate::track::Track;

use std::cell::Cell;

struct SpotifyThings {
    event_manager: EventManager,
    spotify: Arc<spotify::Spotify>,
    queue: Arc<queue::Queue>,
    library: Arc<Library>,
}

struct Init {
    things: RefCell<Result<SpotifyThings, &'static str>>
}

impl Init {
    fn new(creds: Result<Credentials, String>) -> Self {
        Init {things: RefCell::new(SpotifyThings::new(creds))}
    }
    fn re_init(&self, creds: Result<Credentials, String>) {
        self.things.replace(SpotifyThings::new(creds));
    }
}

impl SpotifyThings {
    fn new(credentials: Result<Credentials, String>) -> Result<SpotifyThings, &'static str> {
        let creds = 
        if credentials.is_err() {
            return Err("credentials not ok (yet?)");
        } else {
            credentials.unwrap()
        };
        println!("using credentials: {} {}", creds.username, String::from_utf8(creds.auth_data.clone()).unwrap());
        if !spotify::Spotify::test_credentials(creds.clone()) {
            return Err("testing credentials failed");
        }
        let cfg: crate::config::Config = {
            let path = config::config_path("config.toml");
            crate::config::load_or_generate_default(
                path,
                |_| Ok(crate::config::Config::default()),
                false,
            )
            .unwrap_or_else(|e| {
                eprintln!("{}", e);
                process::exit(1);
            })
        };
        let event_manager = EventManager::new();
        let spotify = Arc::new(spotify::Spotify::new(
            event_manager.clone(),
            creds,
            &cfg
        ));
        let queue = Arc::new(queue::Queue::new(spotify.clone()));
        let library = Arc::new(Library::new(
            &event_manager,
            spotify.clone(),
            cfg.use_nerdfont.unwrap_or(false),
        ));
        Ok(SpotifyThings{event_manager, spotify, queue, library})
    }
}

// fn search_track(query: &String, things: &SpotifyThings) -> Vec<Track> {
fn search_track(query: &String, spotify: Arc<spotify::Spotify>) -> Vec<Track> {
    println!("starting search");

    let mut found_tracks: Vec<Track> = vec![];
    let found_track_count: u32 = if let Some(results) = spotify.search_track(&query, 50, 0_u32) {
            let mut t = results.tracks.items.iter().map(|ft| ft.into()).collect();
            let mut r = &mut found_tracks;

            // if append {
                r.append(&mut t);
            // } else {
                // *r = t;
            // }
            results.tracks.total
        } else {
        0
        };
    println!("found {} tracks", found_track_count);
    // let mut search_view = search::SearchView::new(
        // event_manager.clone(),
        // spotify.clone(),
        // queue.clone(),
        // library.clone(),
    // );
// 
    // search_view.run_search(query);
    println!("search run, printing results");

    // for track in found_tracks {
        // println!("{}", track);
    // }
    // let read_result_tracks = search_view.results_tracks.read();
    // match read_result_tracks {
    //     Ok(tracks) => {
    //         for track in &*tracks {
    //             println!("{}", track);
    //         }
    //     },
    //     Err(x) => {
    //     }
    // };
    println!("done printing results");

    found_tracks

}

struct WindowComponents {
    search_revealer: &'static Revealer,
}

#[derive(Clone)]
struct UiElements {
    play_icon: Rc<RefCell<gtk::Image>>,
    pause_icon: Rc<RefCell<gtk::Image>>,
}

thread_local!(
    static GLOBAL: RefCell<Option<(EventManager, Arc<spotify::Spotify>, Arc<RwLock<gtk::Stack>>, UiElements)>> = RefCell::new(None)
);

lazy_static! {
    static ref spotify_things: Arc<RwLock<Result<SpotifyThings, &'static str>>> = Arc::new(RwLock::new(Err("no credentials yet")));
}

fn build_ui<'a>(application: &gtk::Application) {
    let glade_src = include_str!("spotui.glade");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("window1").expect("Couldn't get window1");
    window.set_application(Some(application));



    let login_stack: Arc<RwLock<gtk::Stack>> = Arc::new(RwLock::new(builder
        .get_object("login_stack")
        .expect("couldn't get login_stack")));
    let login_form: gtk::Grid = builder
        .get_object("login_form")
        .expect("couldn't get login_form");
    let login_ui: Rc<RefCell<gtk::Box>> = Rc::new(RefCell::new(builder
        .get_object("login_ui")
        .expect("couldn't get login_ui")));
    let ui_box: Arc<RwLock<gtk::Box>> = Arc::new(RwLock::new(builder
    // let ui_box: Rc<gtk::Box> = Rc::new(builder
        .get_object("main_ui")
        .expect("couldn't get main_ui")));

    let results_listbox: ListBox = builder
        .get_object("results_listbox")
        .expect("couldn't get results_listbox");

    let search_revealer: Revealer = builder
        .get_object("search_revealer")
        .expect("Couldn't get search_revealer");
        
    let search_combo: gtk::Box = builder
        .get_object("search_combo")
        .expect("Couldn't get search_combo");
    let search_button: gtk::Button = builder
        .get_object("search_button")
        .expect("Couldn't get search_button");
    let play_pause_button: gtk::Button = builder
        .get_object("play_pause_button")
        .expect("couldn't get play_pause_button");
    let play_pause_stack: gtk::Stack = builder
        .get_object("play_pause_stack")
        .expect("Couldn't get play_pause_revealer");
    let pause_icon: Rc<RefCell<gtk::Image>> = Rc::new(RefCell::new(builder
        .get_object("pause_icon")
        .expect("Couldn't get pause_icon")));
    let play_icon: Rc<RefCell<gtk::Image>> = Rc::new(RefCell::new(builder
        .get_object("play_icon")
        .expect("Couldn't get play_icon")));

    let sr: Rc<RefCell<Revealer>> = Rc::new(RefCell::new(search_revealer));
    let pp_stack_arc: Arc<RwLock<gtk::Stack>> = Arc::new(RwLock::new(play_pause_stack));

    let search_box: SearchEntry = builder
        .get_object("search_box")
        .expect("Couldn't get search_box");


    {
        let sr2 = sr.clone();
        search_button.connect_clicked(move |_| {
            sr2.borrow_mut().set_reveal_child(true);
        });
    }

    {
        let sr2 = sr.clone();
        search_box.connect_stop_search(move |sbox: &SearchEntry| {
            sr2.borrow_mut().set_reveal_child(false);
        });
    }

    //let login_button: gtk::Button = builder
    //    .get_object("login_button")
    //    .expect("couldn't get login button");
    let login_button: Rc<RefCell<gtk::Button>> = Rc::new(RefCell::new(builder
        .get_object("login_button")
        .expect("couldn't get login button")));

    let username_entry: Rc<RefCell<gtk::Entry>> = Rc::new(RefCell::new(builder
        .get_object("username_entry")
        .expect("Couldn't get username_entry")));
    let password_entry: Rc<RefCell<gtk::Entry>> = Rc::new(RefCell::new(builder
        .get_object("password_entry")
        .expect("Couldn't get password_entry")));
    let login_error_bar: gtk::InfoBar = builder
        .get_object("login_error_bar")
        .expect("Couldn't get login error bar");
    login_error_bar.set_revealed(false);
    let login_error_bar = Rc::new(RefCell::new(login_error_bar));

    login_error_bar.borrow().connect_close(|bar| {
        bar.set_revealed(false);
    });
    login_error_bar.borrow().connect_response(|bar, response| {
        if response == gtk::ResponseType::Close {
            bar.set_revealed(false);
        }
    });

    let attempting_login: Rc<RefCell<gtk::Box>> = Rc::new(RefCell::new(builder
        .get_object("attempting_login")
        .expect("couldn't get attempting_login (spinner)")));

    login_stack.read().unwrap().set_visible_child(&(*attempting_login.borrow()));

    let search_combo_rc = Arc::new(RwLock::new(search_combo));


    let login_things = Login_Things {search_combo_rc,
        login_stack, ui_box, login_error_bar, login_ui, attempting_login, pp_stack_arc};
    let ui_elements = UiElements {play_icon: play_icon.clone(), pause_icon: pause_icon.clone()};

    {   let username_entry = username_entry.clone();
        let login_things = login_things.clone();
        let ui_elements = ui_elements.clone();
    password_entry.borrow().connect_activate(move |pw| {
        let password = pw.get_text();
        let username = username_entry.borrow().get_text();
        try_login(login_things.clone(), ui_elements.clone(), Ok((username.to_string(), password.to_string())));
    })};

    {   let username_entry = username_entry.clone();
        let login_things = login_things.clone();
        let ui_elements = ui_elements.clone();
    login_button.clone().borrow().connect_clicked(move |_| {
        let password = password_entry.borrow().get_text();
        let username = username_entry.borrow().get_text();
        try_login(login_things.clone(), ui_elements.clone(), Ok((username.to_string(), password.to_string())));
        //login_error_bar.borrow().set_revealed(false);
        //let username = String::from(username_entry.get_text());
        //let password = String::from(password_entry.get_text());
        //println!("re-trying credentials");
        //let credentials = authentication::create_credentials(username, password);
        //if credentials.is_ok() {
        //    let new_things = SpotifyThings::new(credentials);
        //    if new_things.is_ok() {
        //        *spotify_things.write().unwrap() = new_things;
        //        login_stack.read().unwrap().set_visible_child(&(*ui_box.read().unwrap()));
        //        sc_rc_2.read().unwrap().set_visible(true);
        //    }
        //    else {
        //        println!("failed to login with credentials");
        //        login_error_bar.borrow().set_revealed(true);
        //    }
        //} else {
        //    println!("credentials were not ok");
        //}
        //println!("done setting credentials");
        // drop(spotify_things);
    })};


    search_box.connect_activate(move |sbox| {
        println!("searching");

        let spot_things_ref = spotify_things.clone();
        let spot_things = spot_things_ref.read();
        let things = spot_things.as_ref();
        let things = match things {
           Ok(x) => x,
           Err(e) => { println!("not yet initialised"); return(); }
        };
        let mut listbox_row_builder = gtk::ListBoxRowBuilder::new();
        listbox_row_builder = listbox_row_builder.activatable(true);
        println!("searching...");
        let query = String::from(sbox.get_text().as_str());
        let spotify = things.as_ref().unwrap().spotify.clone();
        let results = search_track(&query, spotify.clone());
        // let results = search_track(&query, spotify_things.read().unwrap().spotify.clone());
        //let search_finished = async {
            for child in results_listbox.get_children() {
              //results_listbox.remove(&child);
            }
            //let results: Option<rspotify::model::search::SearchTracks> = search(&query).await;
            for (num, track) in results.iter().enumerate() {
                //let new_entry = &listbox_row_builder.build();
                let new_entry = gtk::ListBoxRow::new();
                let new_entry_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                let entry = gtk::Label::new(Some(&track.title));
                new_entry_box.add(&entry);
                new_entry.add(&new_entry_box);
                new_entry.show_all();
                results_listbox.add(&new_entry);
            }
        let first_result = results.get(0);
        println!("playing first track");
        spotify.load(first_result.unwrap());
        std::thread::sleep(std::time::Duration::from_millis(1000));
        spotify.play();
        println!("volume is {}", spotify.volume());
        //};
    });

    play_pause_button.connect_clicked(move |_| {
        info!("play_pause clicked");
        let spot_things_ref = spotify_things.clone();
        let spot_things = spot_things_ref.read();
        let things = spot_things.as_ref();
        let things = match things {
           Ok(x) => x,
           Err(e) => { println!("not yet initialised"); return(); }
        };
        let spotify = things.as_ref().unwrap().spotify.clone();
        info!("about to play/pause");
        spotify.toggleplayback();
    });


    let controls_revealer: gtk::Revealer = builder.get_object("controls_revealer")
        .expect("failed to get controls_revealer");

    controls_revealer.set_reveal_child(true);

    window.show_all();
    //search_combo_rc.borrow().hide();
    // search_revealer.hide();
    // search_button.hide();
    try_login(login_things.clone(), ui_elements.clone(), Err("no credentials yet".to_string()));
    // attempt to log in
}

#[derive(Clone)]
struct Login_Things {
    search_combo_rc: Arc<RwLock<gtk::Box>>,
    login_stack: Arc<RwLock<gtk::Stack>>,
    ui_box: Arc<RwLock<gtk::Box>>,
    login_error_bar: Rc<RefCell<gtk::InfoBar>>,
    login_ui: Rc<RefCell<gtk::Box>>,
    attempting_login: Rc<RefCell<gtk::Box>>,
    pp_stack_arc: Arc<RwLock<gtk::Stack>>,

}

fn process_spotify_events() -> glib::Continue {

    GLOBAL.with(|global| {
    if let Some((ref event_manager, ref spotify, ref pp_stack_arc, ref ui_elements)) = *global.borrow() {
        for event in event_manager.msg_iter() {
            match event {
                Event::Player(state) => {
                    trace!("event received: {:?}", state);
                    spotify.update_status(state.clone());

                    let pp_stack = pp_stack_arc.read().unwrap();
                    match state {
                        PlayerEvent::Playing => {
                            pp_stack.set_visible_child(&(*ui_elements.pause_icon.borrow()));
                        },
                        PlayerEvent::Paused=> {
                            pp_stack.set_visible_child(&(*ui_elements.play_icon.borrow()));
                        },
                        PlayerEvent::Stopped=> {
                        },
                        PlayerEvent::FinishedTrack=> {
                        },

                    }

                    // #[cfg(feature = "mpris")]
                    // mpris_manager.update();

                    // if state == PlayerEvent::FinishedTrack {
                        // queue.next(false);
                    // }
                }
            }
        }
    }});
    // hack for the moment to make sure we don't lose anything
    thread::spawn(move || {
        thread::sleep_ms(16);
        glib::idle_add(process_spotify_events);
    });
    glib::Continue(false)
}

fn try_login(things: Login_Things, ui_elements: UiElements, auth: Result<(String, String), String>) {
    let search_combo_rc = things.search_combo_rc;
    let login_stack = things.login_stack;
    let ui_box = things.ui_box;
    let login_error_bar = things.login_error_bar;
    let login_ui = things.login_ui;
    let attempting_login = things.attempting_login;
    let pp_stack_arc = things.pp_stack_arc;

    let creds_supplied = auth.is_ok();
    let (username, password) = 
    if creds_supplied {
        auth.unwrap()
    } else { ("".to_string(), "".to_string())};

    login_stack.read().unwrap().set_visible_child(&(*attempting_login.borrow()));
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // login thread
    thread::spawn( move || {
        tx.send({
            let credentials = 
            if creds_supplied {
                authentication::create_credentials(username, password)
            } else {
                authentication::try_credentials()
            };
            let mut success = false;
            let things =
            if credentials.is_ok() {
                println!("credentials were ok");
                if spotify::Spotify::test_credentials(credentials.clone().unwrap()) {
                    println!("tested credentials passed");
                    success = true;
                    SpotifyThings::new(credentials)
                } else {
                    Err("couldn't log in with credentials")
                }
            } else {
                Err("failed to read credentials")
            };
            // let tmp = (things, success)
            *(spotify_things.clone().write().unwrap()) = things;
            success
        });
    });
    rx.attach(None, move |success| {
        info!("login result received");
        if success {
            info!("login succeeded");
            login_stack.read().unwrap().set_visible_child(&(*ui_box.read().unwrap()));
            search_combo_rc.read().unwrap().set_visible(true);
            // start event loop
            {
            let spot_things_ref = spotify_things.clone();
            let spot_things = spot_things_ref.read();
            let things = spot_things.as_ref();
            let things = match things {
               Ok(x) => x,
               Err(e) => { println!("not yet initialised"); panic!(); }
            };
            let spotify = things.as_ref().unwrap().spotify.clone();
            let event_manager = things.as_ref().unwrap().event_manager.clone();
            let pp_stack_arc = pp_stack_arc.clone();
            let ui_elements = ui_elements.clone();
            GLOBAL.with(move |global| {
                *global.borrow_mut() = Some((event_manager, spotify, pp_stack_arc, ui_elements))
            });
            glib::idle_add(process_spotify_events);
            }
        } else {
            warn!("login failed");
            login_stack.read().unwrap().set_visible_child(&(*login_ui.borrow()));
            search_combo_rc.read().unwrap().set_visible(true);
            login_error_bar.borrow().set_revealed(true);
        }
        glib::Continue(true)
    });
}

fn sbox_stop_search<'a>(sbox: &'a gtk::SearchEntry, search_revealer: &'a Revealer) {
    search_revealer.set_reveal_child(false);
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        //.chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
// static init: Init = Init { things: RefCell::new(Err("not initialised yet")) };
fn main() {
    setup_logger();
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.builder_basics"),
        Default::default(),
    )
    .expect("Initialization failed...");

    let credentials = authentication::try_credentials();
    // init.re_init(credentials);
    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
