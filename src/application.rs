use glib::Continue;
use gtk::{
    ApplicationWindow, ContainerExt, DialogExt, DialogFlags, EntryExt, GtkWindowExt, MessageType,
    WidgetExt,
};
use std::{
    sync::{Arc, Mutex},
    thread::{sleep, spawn},
    time::Duration,
    unreachable,
};

use crate::{
    listener::{Listener, SongInfo},
    lyrics::{happi::HappiLyrics, LyricsFetcher},
    settings::Settings,
    widgets::{HeaderBar, LyricsView},
};

pub struct LyricsApplication {
    window: gtk::ApplicationWindow,
    headerbar: HeaderBar,
    lyrics_view: LyricsView,
    song_info: SongInfo,
}

impl LyricsApplication {
    pub fn new(app: &gtk::Application) -> Self {
        let window = ApplicationWindow::new(app);

        let mut app = LyricsApplication {
            window: window.clone(),
            headerbar: HeaderBar::new(window),
            lyrics_view: LyricsView::new(),
            song_info: SongInfo::default(),
        };

        app.build_ui();

        app
    }

    pub fn build_ui(&mut self) {
        self.window.set_border_width(10);
        self.window.set_position(gtk::WindowPosition::Center);
        self.window.set_size_request(550, 700);

        self.window.set_titlebar(Some(&self.headerbar.container));

        self.window.add(self.lyrics_view.as_widget());

        self.update(SongInfo::default());

        self.window.show_all();
    }

    pub fn mount_listener(self) {
        let song_info = Arc::from(Mutex::from(SongInfo::default()));

        self.check_api_key(Arc::clone(&song_info));

        self.start_update_listener(song_info);
    }

    /**
     * Checks and updates if detected song is changed
     */
    fn start_update_listener(mut self, song_info: Arc<Mutex<SongInfo>>) {
        glib::timeout_add_local(250, move || {
            match song_info.try_lock() {
                Ok(song_info) => {
                    if song_info.song_title != self.song_info.song_title
                        || song_info.artist_name != self.song_info.artist_name
                    {
                        self.update((*song_info).clone());
                    }
                }
                Err(e) => println!("Error: {:?}", e),
            }

            Continue(true)
        });
    }

    fn check_api_key(&self, song_info: Arc<Mutex<SongInfo>>) {
        let settings = Settings::new();

        let api_key = settings.get_api_key();
        let has_api_key = api_key != "";

        if !has_api_key {
            // display api key dialog
            let dialog = gtk::MessageDialog::new(
                Some(&self.window),
                DialogFlags::DESTROY_WITH_PARENT,
                MessageType::Question,
                gtk::ButtonsType::Ok,
                "An api key from happi.dev is needed to retrieve lyrics",
            );

            let api_label = gtk::Entry::new();
            let dialog_box = dialog.get_content_area();

            dialog_box.add(&api_label);

            dialog.connect_response(|d, r| {
                println!("r: {:?}", r);
                match r {
                    gtk::ResponseType::Ok => {
                        d.emit_close();
                    }
                    _ => unreachable!(),
                }
            });

            dialog.connect_close(move |_| {
                let api_key = api_label.get_text();
                Self::start_listening(Arc::clone(&song_info), api_key.as_str());

                let settings = Settings::new();
                settings.set_api_key(api_label.get_text().as_str());
            });

            dialog.show_all();
        } else {
            let settings = Settings::new();
            let api_key = settings.get_api_key();
            Self::start_listening(Arc::clone(&song_info), api_key.as_str());
        }
    }

    /**
     * Listens currently played song. If it changes it retrieves its lyrics as well
     */
    fn start_listening(song_info: Arc<Mutex<SongInfo>>, api_key: &str) {
        let api_key: String = api_key.into();

        spawn(move || {
            let mut listen = Listener::new();

            listen.connect_signal_blocking(Arc::clone(&song_info));

            let lyrics_fetcher = HappiLyrics::new(api_key);

            {
                let song_info = Arc::clone(&song_info);
                // Listen to spotify changes
                loop {
                    listen.listen();

                    let song_info = song_info.try_lock();

                    match song_info {
                        Ok(mut song_info) => {
                            if song_info.pull_lyrics.is_none() {
                                println!(
                                    "Changed to: {} - {}",
                                    song_info.song_title, song_info.artist_name
                                );
                                let lyrics = lyrics_fetcher
                                    .get_lyrics(&song_info.song_title, &song_info.artist_name);

                                song_info.pull_lyrics = Some(
                                    lyrics
                                        .map(|l| l.lyrics)
                                        .unwrap_or("Lyrics not available".into()),
                                );
                            }
                        }
                        Err(e) => println!("Error: {:?}", e),
                    }

                    sleep(Duration::from_millis(250));
                }
            }
        });
    }

    pub fn update(&mut self, song_info: SongInfo) {
        self.song_info = song_info;
        self.lyrics_view.update(&self.song_info);
    }
}
