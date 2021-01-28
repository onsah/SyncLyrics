use application::LyricsApplication;
use gio::prelude::*;
use gtk::prelude::*;

use style::STYLE;

mod application;
mod listener;
mod lyrics;
mod style;
mod widgets;

#[tokio::main]
async fn main() {
    let application = gtk::Application::new(
        Some("com.github.onsah.sync-lyrics"),
        gio::ApplicationFlags::empty(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        setup_style();

        let application = LyricsApplication::new(app);

        application.mount_listener();
    });

    application.run(&[]);
}

fn setup_style() {
    // css
    let provider = gtk::CssProvider::new();
    provider.load_from_data(STYLE.as_bytes()).unwrap();

    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
