use application::LyricsApplication;
use gio::prelude::*;

mod app_state;
mod application;
mod configs;
mod spotify_listener;
mod lyrics;
mod widgets;
mod utils;

#[tokio::main]
async fn main() {
    let application = gtk::Application::new(
        Some("com.github.onsah.sync-lyrics"),
        gio::ApplicationFlags::empty(),
    );

    application.connect_activate(|app| {
        setup_style();

        LyricsApplication::new(app)
            .mount_listener();
    });

    application.run();
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
