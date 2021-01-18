

use application::LyricsApplication;
use gtk::prelude::*;
use gio::prelude::*;

use style::STYLE;

mod application;
mod lyrics;
mod listener;
mod widgets;
mod style;
mod settings;

fn main() {
    let application =
        gtk::Application::new(Some("com.github.onsah.sync-lyrics"), gio::ApplicationFlags::empty())
            .expect("Initialization failed...");

    application.connect_startup(|app| {
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