use gdk::prelude::ActionGroupExt;
use glib::{Continue};
use gtk::{ApplicationWindow};
use std::{sync::Arc, time::Duration};
use gtk::prelude::{GtkWindowExt, WidgetExt};
use tokio::{sync::Mutex, time::sleep};

use crate::{app_state::AppState, spotify_listener::SpotifyListener, lyrics::{LyricsError, genius::Genius}, utils::spawn_as_abortable, widgets::{HeaderBar, LyricsView}};

pub struct LyricsApplication {
    window: gtk::ApplicationWindow,
    headerbar: HeaderBar,
    lyrics_view: LyricsView,
    app_state: AppState,
}

impl LyricsApplication {
    pub fn new(app: &adw::Application) -> Self {
        let window = ApplicationWindow::new(app);

        window.present();

        let mut app = LyricsApplication {
            window: window.clone(),
            headerbar: HeaderBar::new(window),
            lyrics_view: LyricsView::new(),
            app_state: AppState::Connecting,
        };

        app.build_ui();

        app
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

    pub fn mount_listener(self) {
        let song_info = Arc::from(Mutex::from(AppState::Connecting));

        self.connect_spotify_listener_loop(Arc::clone(&song_info));

        self.connect_ui_updater_loop(song_info);
    }

    /**
     * Starts the loop so that it will be killed when app exits
     */
    fn connect_spotify_listener_loop(&self, song_info: Arc<Mutex<AppState>>) {
        // This allows aborting it when window is closed
        let abort_handle = spawn_as_abortable(Self::spotify_listener_loop(song_info));

        // Terminate the future when window is closed
        self.window.connect_action_removed(None, move |_, _| {
            abort_handle.abort();
        });
    }

    // Listen to spotify changes. If it changes it retrieves its lyrics as well
    async fn spotify_listener_loop(app_state: Arc<Mutex<AppState>>) {
        let mut spotify_listener = SpotifyListener::new();

        while !spotify_listener.connect_signal_loop(Arc::clone(&app_state)) {
            sleep(Duration::from_secs(500)).await;
        }

        let mut lyrics_fetcher = Genius::new();

        {
            let app_state = Arc::clone(&app_state);
            loop {
                spotify_listener.listen();

                let app_state_guard = app_state.lock().await;

                if let AppState::FetchingLyrics {
                    song_name,
                    artist_name,
                } = &*app_state_guard
                {
                    println!("Changed to: {} - {}", song_name, artist_name);

                    let (song_name, artist_name) = (song_name.to_string(), artist_name.to_string());

                    // No need to lock during web request
                    drop(app_state_guard);

                    let lyrics = lyrics_fetcher.get_lyrics(&song_name, &artist_name).await;

                    let mut app_state_guard = app_state.lock().await;

                    *app_state_guard = match lyrics {
                        Ok(response) => AppState::LyricsFetched {
                            song_name,
                            artist_name,
                            lyrics: response.lyrics,
                            cover_art: Some(response.cover_art),
                        },
                        Err(err) => match err {
                            LyricsError::Network(_) => AppState::NetworkFailed,
                            LyricsError::SongNotFound {
                                song_name, artist
                            } => AppState::SongNotFound {
                                song_name, artist_name: artist
                            },
                            _ => todo!("handle error {:?}", err)
                        }
                    };

                } else {
                    // no lyrics to be pulled, can sleep a bit
                    sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }

    /**
     * Checks and updates if detected song is changed
     * Can't make async because gtk widgets are not Send
     */
     fn connect_ui_updater_loop(mut self, song_info: Arc<Mutex<AppState>>) {
        glib::timeout_add_local(Duration::from_millis(50), move || {
            match song_info.try_lock() {
                Ok(song_info) => {
                    self.update_ui((*song_info).clone());
                }
                Err(_) => (/* println!("update_listener: {:?}", e) */),
            }

            Continue(true)
        });
    }

    pub fn update_ui(&mut self, new_app_state: AppState) {
        match &new_app_state {
            AppState::LyricsFetched { lyrics, cover_art, .. } => {
                if !self.app_state.fetched() {
                    self.lyrics_view.song_data_retrieved(lyrics, cover_art.as_deref());
                }
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
