use glib::{Continue};
use gtk::{ApplicationWindow};
use std::{time::Duration, rc::Rc};
use crossbeam_channel::Receiver;
use gtk::prelude::{GtkWindowExt, WidgetExt};

use crate::{app_state::AppState, widgets::{HeaderBar, LyricsView}, ui::UIEvent};

pub struct LyricsApplication {
    window: gtk::ApplicationWindow,
    headerbar: HeaderBar,
    lyrics_view: LyricsView,
    app_state: AppState,
}

impl LyricsApplication {
    pub fn init(app: &adw::Application, ui_event_receiver: Rc<Receiver<UIEvent>>) {
        let window = ApplicationWindow::new(app);

        window.present();

        let mut app = LyricsApplication {
            window: window.clone(),
            headerbar: HeaderBar::new(window),
            lyrics_view: LyricsView::new(),
            app_state: AppState::Connecting,
        };

        app.build_ui();

        app.init_ui_event_consumer(ui_event_receiver);
    }

    pub fn build_ui(&mut self) {
        // self.window.set_border_width(0);
        // self.window.set_position(gtk::WinowPosition::Center);
        self.window.set_resizable(false);
        self.window.set_hexpand(false);
        self.window.set_vexpand(false);

        self.window.set_titlebar(Some(&self.headerbar.container));

        self.window.set_child(Some(self.lyrics_view.as_widget()));
    }

    // TODO: Make this not take self
    fn init_ui_event_consumer(mut self, ui_event_receiver: Rc<Receiver<UIEvent>>) {
        glib::timeout_add_local(Duration::from_millis(50), move || {
            match ui_event_receiver.try_recv() {
                Ok(ui_event) => {
                    match ui_event {
                        UIEvent::SongInformationFetched { 
                            song_name, 
                            artist_name, 
                            lyrics, 
                            cover_art 
                        } => self.update_ui(AppState::LyricsFetched { 
                            song_name, 
                            artist_name, 
                            lyrics, 
                            cover_art: Some(cover_art) 
                        }),
                        UIEvent::FetchingLyrics { song_name, artist_name } => self.update_ui(
                            AppState::FetchingLyrics { song_name, artist_name }
                        )
                    }
                }
                Err(_) => (/* println!("update_listener: {:?}", e) */),
            }

            Continue(true)
        });
    }

    pub fn update_ui(&mut self, new_app_state: AppState) {
        match &new_app_state {
            AppState::LyricsFetched { lyrics, cover_art, .. } => {
                self.lyrics_view.song_data_retrieved(lyrics, cover_art.as_deref());
            }
            AppState::FetchingLyrics {
                song_name,
                artist_name,
            } => {
                let should_change = self.app_state.is_different(song_name, artist_name);

                if should_change {
                    self.lyrics_view.song_changed(song_name, artist_name);
                }
            }
            AppState::NetworkFailed => self.lyrics_view.network_failed(),
            AppState::SongNotFound {
                song_name, artist_name
            } => {
                self.lyrics_view.song_not_found(song_name, artist_name);
            }
            AppState::Connecting => (),
        }

        self.app_state = new_app_state;
    }
}
