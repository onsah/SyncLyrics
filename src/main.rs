use std::{thread::{spawn, JoinHandle}, rc::Rc};

use application::LyricsApplication;
use gdk::{prelude::{ApplicationExt, ApplicationExtManual}, gio::ApplicationFlags};
use lyrics::genius::Genius;
use spotify::{spotify_event_producer::SpotifyEventProducer, spotify_event::SpotifyEvent};
use tokio::runtime::Runtime;
use crossbeam_channel::Receiver;
use ui::UIEvent;

mod app_state;
mod application;
mod configs;
mod spotify;
mod lyrics;
mod widgets;
mod utils;
mod ui;

#[tokio::main]
async fn main() {
    let application = adw::Application::new(
        Some("com.github.onsah.sync-lyrics"),
        ApplicationFlags::empty(),
    );

    let (spotify_event_sender, spotify_event_receiver) = crossbeam_channel::unbounded::<SpotifyEvent>();

    let (fl_spotify_event_sender, fl_spotify_event_receiver) = crossbeam_channel::unbounded();

    // TODO: Close thread on app close
    let _spotify_event_producer = SpotifyEventProducer::init(spotify_event_sender);

    let (ui_event_sender, ui_event_receiver) = crossbeam_channel::unbounded();

    fetch_lyrics(fl_spotify_event_receiver, ui_event_sender.clone());

    // Have to do this stupid thing, 
    // Because connect_activate is Fn and not FnOnce
    // Which means can't capture stuff by move since it can be
    // called multiple times.
    let ui_event_receiver = Rc::from(ui_event_receiver);

    spawn(move || {
        loop {
            let event = spotify_event_receiver.recv().unwrap();
            fl_spotify_event_sender.send(event.clone()).unwrap();
            let SpotifyEvent::SongChanged { song_name, artist_name } = event;
            ui_event_sender.send(
                UIEvent::FetchingLyrics {
                    song_name, artist_name,
                }
            ).unwrap();
        }
    });

    application.connect_activate(move |app| {
        setup_style();

        LyricsApplication::new(app)
            .init_ui_event_consumer(ui_event_receiver.clone());
    });

    application.run();
}

fn fetch_lyrics(spotify_event_receiver: Receiver<SpotifyEvent>, ui_event_sender: crossbeam_channel::Sender<UIEvent>) -> JoinHandle<()> {
    spawn(move || {
        let mut lyrics_fetcher = Genius::new();
        let rt = Runtime::new().unwrap();
        loop {
            let spotify_event = spotify_event_receiver.recv().unwrap();
            
            match spotify_event {
                SpotifyEvent::SongChanged { song_name, artist_name } => {
                    let lyrics_result = rt.block_on(
                        lyrics_fetcher.get_lyrics(&song_name, &artist_name)
                    ).unwrap();

                    ui_event_sender.send(UIEvent::SongInformationFetched { 
                        song_name, 
                        artist_name, 
                        lyrics: lyrics_result.lyrics, 
                        cover_art: lyrics_result.cover_art, 
                    }).unwrap();
                }
            }
        }
    })
}

fn setup_style() {
    // css
    // I am thinking of not using custom css at all.
    /* let provider = gtk::CssProvider::new();
    provider.load_from_data(STYLE.as_bytes()).unwrap();

    gtk::StyleContext::add_provider_for_display(
        // TODO: add gdk4
        &gdk::Display::default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    ); */
}
