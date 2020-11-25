//! # Builder Basics Sample
//!
//! This sample demonstrates how to use the builder with an imported glade file

extern crate gio;
extern crate gtk;

use std::cell::RefCell;
use std::rc::Rc;

use gio::prelude::*;
use gtk::prelude::*;

use gtk::SearchEntryExt;
use gtk::{ApplicationWindow, Builder, Button, ListBox, Revealer, SearchEntry};

use std::env;
use std::env::args;
//use tokio_compat::prelude::*;
use tokio_core::reactor::Core;

//use librespot::core::authentication::Credentials;
//use librespot::core::config::SessionConfig;
//use librespot::core::session::Session;
//use librespot::core::spotify_id::SpotifyId;
//use librespot::metadata::{Metadata, Playlist, Track};
//use librespot::playback::config::PlayerConfig;
//
//use env_logger;
//use librespot::playback::audio_backend;
//use librespot::playback::player::Player;
//
//
//use rspotify::client::Spotify;
//use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth};
//use rspotify::senum::Country;
//use rspotify::util::get_token;

//use futures::executor::block_on;
//use futures::join;
//
//use std::path::Path;
//
//use librespot_core::authentication::Credentials as RespotCredentials;
//use librespot_protocol::authentication::AuthenticationType;

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
//fn main() {}

//#[tokio::main]
//async fn main() {
//    let search_finished = async {
//        search().await;
//    };
//
//    block_on(search_finished);
//}

fn get_credentials(reset: bool) -> Credentials {
    let path = config::config_path("credentials.toml");
    if reset && fs::remove_file(&path).is_err() {
        error!("could not delete credential file");
    }

    let creds = authentication::create_credentials().unwrap();

    //let creds = match crate::config::load_or_generate_default(&path, authentication::create_credentials, true) {
    //    Ok(x) => x,
    //    Err(e) => {
    //    },
    //};

    // #[cfg(target_family = "unix")]
    // std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o600))
    //     .unwrap_or_else(|e| {
    //         eprintln!("{}", e);
    //         process::exit(1);
    //     });

    creds
}

fn search_track(query: &String) -> Vec<Track> {
    println!("starting search");
    let mut credentials = get_credentials(false);


    // Things here may cause the process to abort; we must do them before creating curses windows
    // otherwise the error message will not be seen by a user
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
        credentials,
        &cfg,
    ));
    let queue = Arc::new(queue::Queue::new(spotify.clone()));
    let library = Arc::new(Library::new(
        &event_manager,
        spotify.clone(),
        cfg.use_nerdfont.unwrap_or(false),
    ));


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

    // old version here
    // Set client_id and client_secret in .env file or
    // export CLIENT_ID="your client_id"
    // export CLIENT_SECRET="secret"
    // export REDIRECT_URI=your-direct-uri

    // Or set client_id, client_secret,redirect_uri explictly
    //let mut oauth = SpotifyOAuth::default()
    //    .client_id("33275c3ad21a4781bf4facf3a1e0e778")
    //    .client_secret("28a15495fcbe4175abce0d36034514eb")
    //    .redirect_uri("https://caelus.nz/spotify_auth")
    //    .build();

    ////let mut oauth = SpotifyOAuth::default().scope("user-read-private").build();
    //match get_token(&mut oauth).await {
    //    Some(token_info) => {
    //        //let client_credential = SpotifyClientCredentials::default()
    //        //    .token_info(token_info)
    //        //    .build();
    //        // Or set client_id and client_secret explictly
    //        let client_credential = SpotifyClientCredentials::default()
    //            .client_id("e1dce60f1e274e20861ce5d96142a4d3")
    //            .client_secret("23a3a18423b14bf383ce46c8ee271094")
    //            .build();
    //        let spotify = Spotify::default()
    //            .client_credentials_manager(client_credential)
    //            .build();
    //        //let query = "abba";
    //        let result = spotify
    //            .search_track(query, 10, 0, Some(Country::UnitedStates))
    //            .await;
    //        println!("search result:{:?}", result);
    //        let actual_result = match result {
    //            Ok(x) => x,
    //            Err(e) => {
    //                panic!(e);
    //            }
    //        };
    //        for (num, track) in actual_result.tracks.items.iter().enumerate() {
    //            println!("{}, {}, {}", num, track.name, track.id.as_ref().unwrap());
    //        }
    //        return Some(actual_result);
    //        //println!("playing item 0");
    //        //let track_to_play = actual_result.tracks.items.first();
    //        //play_track(track_to_play.unwrap().id.as_ref().unwrap());
    //    }
    //    None => println!("auth failed"),
    //};
    //None
}

//fn play_track(track_id: &String) {
//    env_logger::init();
//    let mut core = Core::new().unwrap();
//    let handle = core.handle();
//
//    let session_config = SessionConfig::default();
//
//    //let args: Vec<_> = env::args().collect();
//    //if args.len() != 4 {
//    //    println!("Usage: {} USERNAME PASSWORD PLAYLIST", args[0]);
//    //}
//    //let username = args[1].to_owned();
//    //let password = args[2].to_owned();
//    //let credentials = Credentials::with_password(username, password);
//
//    //let uri_split = args[3].split(":");
//    //let uri_parts: Vec<&str> = uri_split.collect();
//    //println!("{}, {}, {}", uri_parts[0], uri_parts[0], uri_parts[0]);
//
//    //let plist_uri = SpotifyId::from_base62(uri_parts[0]).unwrap();
//
//    //let session = core
//    //    .run(Session::connect(session_config, credentials, None, handle))
//    //    .unwrap();
//
//    //let plist = core.run(Playlist::get(&session, plist_uri)).unwrap();
//    //println!("{:?}", plist);
//    //for track_id in plist.tracks {
//    //    let plist_track = core.run(Track::get(&session, track_id)).unwrap();
//    //    println!("track: {} ", plist_track.name);
//    //}
//
//    let player_config = PlayerConfig::default();
//
//    let args: Vec<_> = env::args().collect();
//    if args.len() != 4 {
//        println!("Usage: {} USERNAME PASSWORD TRACK", args[0]);
//    }
//    //let username = args[1].to_owned();
//    //let password = args[2].to_owned();
//    let username = "uesrname".to_owned();
//    let password = "password".to_owned();
//    let credentials = Credentials::with_password(username, password);
//
//    let track = SpotifyId::from_base62(&track_id).unwrap();
//
//    let backend = audio_backend::find(None).unwrap();
//
//    println!("Connecting ..");
//    let session = core
//        .run(Session::connect(session_config, credentials, None, handle))
//        .unwrap();
//
//    let (mut player, _) = Player::new(player_config, session.clone(), None, move || {
//        (backend)(None)
//    });
//
//    let receiver = player.load(track, true, 0);
//
//    println!("Playing...");
//    player.play();
//    //core.run(receiver).unwrap();
//    core.run((player.get_end_of_track_future()));
//    //tokio_compat::run((player.get_end_of_track_future()).unwrap();
//
//    println!("Done");
//}

struct WindowComponents {
    search_revealer: &'static Revealer,
}

fn build_ui<'a>(application: &gtk::Application) {
    let glade_src = include_str!("spotui.glade");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("window1").expect("Couldn't get window1");
    window.set_application(Some(application));
    let bigbutton: Button = builder.get_object("button1").expect("Couldn't get button1");

    let results_listbox: ListBox = builder
        .get_object("results_listbox")
        .expect("couldn't get results_listbox");

    let search_revealer: Revealer = builder
        .get_object("search_revealer")
        .expect("Couldn't get search_revealer");

    let sr: Rc<RefCell<Revealer>> = Rc::new(RefCell::new(search_revealer));

    let search_box: SearchEntry = builder
        .get_object("search_box")
        .expect("Couldn't get search_box");

    {
        let sr2 = sr.clone();
        bigbutton.connect_clicked(move |_| {
            sr2.borrow_mut().set_reveal_child(true);
        });
    }

    {
        let sr2 = sr.clone();
        search_box.connect_stop_search(move |sbox: &SearchEntry| {
            sr2.borrow_mut().set_reveal_child(false);
        });
    }
    //search_box.connect_stop_search(|sbox: &SearchEntry| {
    //    search_revealer.set_reveal_child(false);
    //});
    //search_box.connect_stop_search(|sbox| sbox_stop_search(&sbox, &search_revealer));

    search_box.connect_activate(move |sbox| {
        let mut listbox_row_builder = gtk::ListBoxRowBuilder::new();
        listbox_row_builder = listbox_row_builder.activatable(true);
        println!("searching...");
        let query = String::from(sbox.get_text().as_str());
        let results = search_track(&query);
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
        //};
        // block_on(search_finished);
        //let new_entry_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        //new_entry_box.add(&new_entry_label);
        //new_entry.add(&new_entry_box);
        //results_listbox.add(&new_entry);
        ////results_listbox.show_all();
        //new_entry.show_all();
    });

    window.show_all();
}

fn sbox_stop_search<'a>(sbox: &'a gtk::SearchEntry, search_revealer: &'a Revealer) {
    search_revealer.set_reveal_child(false);
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.builder_basics"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
