use std::{sync::{Arc, Mutex}, thread::{sleep, spawn}, time::Duration, unreachable};

use glib::Continue;
use gtk::{Adjustment, ApplicationWindow, ContainerExt, DialogExt, DialogFlags, EntryExt, GtkWindowExt, LabelExt, MessageType, WidgetExt};

use crate::{listener::{Listener, SongInfo}, lyrics::{LyricsFetcher, happi::HappiLyrics}, settings::Settings, widgets::HeaderBar};

pub struct LyricsApplication {
    window: gtk::ApplicationWindow,
    headerbar: HeaderBar,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    lyrics_label: gtk::Label,
}

impl LyricsApplication {
    pub fn new(app: &gtk::Application) -> Self {
        let window = ApplicationWindow::new(app);

        let mut app = LyricsApplication {
            window: window.clone(),
            headerbar: HeaderBar::new(window),
            title_label: gtk::Label::new (Some("")),
            artist_label: gtk::Label::new(Some("")),
            lyrics_label: gtk::Label::new(Some("")),
        };
        
        app.build_ui();

        app
    }

    pub fn build_ui(&mut self) {
        self.window.set_border_width(10);
        self.window.set_position(gtk::WindowPosition::Center);
        self.window.set_size_request(550, 700);

        self.window.set_titlebar(Some(&self.headerbar.container));
        
        self.title_label.set_widget_name("title1");
        self.lyrics_label.set_line_wrap(true);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
        vbox.add(&self.title_label);
        vbox.add(&self.artist_label);

        // Lyrics label is scrolled
        let label_scroller = gtk::ScrolledWindow::new(None as Option<&Adjustment>, None as Option<&Adjustment>);
        label_scroller.set_size_request(250, 250);
        label_scroller.set_vexpand(true);
        label_scroller.add(&self.lyrics_label);

        vbox.add(&label_scroller);
        self.window.add(&vbox);

        self.update(&SongInfo::default());

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
                    self.update(&song_info);
                },
                Err(e) => println!("Error: {:?}", e),
            }

            Continue(true)
        });
    }

    fn check_api_key(&self, song_info: Arc<Mutex<SongInfo>>) {
        let settings = Settings::new();

        let api_key = settings.get_api_key();
        println!("api key: {}", api_key);
        let has_api_key = api_key != "";
        
        if !has_api_key {
            // display api key dialog
            let dialog = gtk::MessageDialog::new(
                Some(&self.window),
                DialogFlags::DESTROY_WITH_PARENT,
                MessageType::Question,
                gtk::ButtonsType::Ok,
                "An api key from happi.dev is needed to retrieve lyrics"
            );

            let api_label = gtk::Entry::new();
            let dialog_box = dialog.get_content_area();

            dialog_box.add(&api_label);  

            dialog.connect_response(|d, r| {
                println!("r: {:?}", r);
                match r {
                    gtk::ResponseType::Ok => {
                        d.emit_close();
                    },
                    _ => unreachable!()
                }
            });

            dialog.connect_close(move |_| {
                let api_key = api_label.get_text();
                Self::start_listening_spotify(Arc::clone(&song_info), api_key.as_str());

                let settings = Settings::new();
                settings.set_api_key(api_label.get_text().as_str());
            });

            dialog.show_all();
        } else {
            let settings = Settings::new();
            let api_key = settings.get_api_key();
            Self::start_listening_spotify(Arc::clone(&song_info), api_key.as_str());
        }
    }

    fn start_listening_spotify(song_info: Arc<Mutex<SongInfo>>, api_key: &str) {

        let mut listen = Listener::new();

        listen.connect_signal(Arc::clone(&song_info));
        
        println!("Api key: {}", api_key);
        let lyrics_fetcher = HappiLyrics::new(api_key.into());
    
        {
            let song_info = Arc::clone(&song_info);
            // Listen to spotify changes
            spawn(move || {
                loop { 
                    listen.listen();
        
                    let song_info = song_info.try_lock();
        
                    match song_info {
                        Ok(mut song_info) => {
                            if song_info.pull_lyrics.is_none() {
                                println!("Changed to: {} - {}", song_info.song_title, song_info.artist_name);
                                let lyrics = lyrics_fetcher.get_lyrics(&song_info.song_title, &song_info.artist_name);
        
                                song_info.pull_lyrics = Some(lyrics.map(|l| l.lyrics).unwrap_or("Lyrics not available".into()));
                            }
                        },
                        Err(e) => println!("Error: {:?}", e),
                    }
        
                    sleep(Duration::from_millis(250));
                }
            });
        }
    }

    pub fn update(&mut self, song_info: &SongInfo) {
        self.set_song_title(&song_info.song_title);
        self.set_artist(&song_info.artist_name);
        self.set_lyrics(song_info.pull_lyrics.as_ref().map(String::as_str).unwrap_or("lyrics are not available"));
    }

    fn set_song_title(&mut self, song_title: &str) {
        self.title_label.set_markup(&format!("<span size=\"x-large\">{}</span>", song_title));
    }

    fn set_artist(&mut self, artist_name: &str) {
        self.artist_label.set_markup(&format!("<span size=\"large\" weight=\"bold\">{}</span>", artist_name));
    }

    fn set_lyrics(&mut self, lyrics: &str) {
        self.lyrics_label.set_markup(&format!("<span size=\"large\">{}</span>", lyrics));
    }
}