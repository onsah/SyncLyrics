use futures::{
    executor,
    future::{AbortHandle, Abortable},
    Future,
};
use glib::Continue;
use gtk::{ApplicationWindow, ContainerExt, GtkWindowExt, Inhibit, WidgetExt};
use std::{borrow::Borrow, ops::Deref, sync::Arc, time::Duration, unreachable};
use tokio::{sync::Mutex, time::sleep};

use crate::{
    app_state::AppState,
    listener::Listener,
    lyrics::genius::Genius,
    widgets::{HeaderBar, LyricsView},
};

pub struct LyricsApplication {
    window: gtk::ApplicationWindow,
    headerbar: HeaderBar,
    lyrics_view: LyricsView,
    app_state: AppState,
}

impl LyricsApplication {
    pub fn new(app: &gtk::Application) -> Self {
        let window = ApplicationWindow::new(app);

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
        self.window.set_border_width(0);
        self.window.set_position(gtk::WindowPosition::Center);
        self.window.set_resizable(false);
        self.window.set_hexpand(false);
        self.window.set_vexpand(false);

        self.window.set_titlebar(Some(&self.headerbar.container));

        self.window.add(self.lyrics_view.as_widget());

        self.window.show_all();
    }

    pub fn mount_listener(self) {
        let song_info = Arc::from(Mutex::from(AppState::Connecting));

        self.song_info_start_listening(Arc::clone(&song_info));

        self.start_update_listener(song_info);
    }

    /**
     * Checks and updates if detected song is changed
     * Can't make async because gtk widgets are not Send
     */
    fn start_update_listener(mut self, song_info: Arc<Mutex<AppState>>) {
        glib::timeout_add_local(50, move || {
            match song_info.try_lock() {
                Ok(song_info) => {
                    self.update((*song_info).clone());
                }
                Err(_) => (/* println!("update_listener: {:?}", e) */),
            }

            Continue(true)
        });
    }

    /**
     * Listens currently played song. If it changes it retrieves its lyrics as well
     */
    fn song_info_start_listening(&self, song_info: Arc<Mutex<AppState>>) {
        // This allows aborting it when window is closed
        let abort_handle = Self::spawn_as_abortable(Self::song_info_listener_loop(song_info));

        // Terminate the future when window is closed
        self.window.connect_delete_event(move |_, _| {
            abort_handle.abort();
            Inhibit(false)
        });
    }

    fn spawn_as_abortable<F: Future + Send + 'static>(fut: F) -> AbortHandle
    where
        <F as Future>::Output: Send,
    {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();

        tokio::spawn(Abortable::new(fut, abort_registration));

        abort_handle
    }

    // Listen to spotify changes
    async fn song_info_listener_loop(app_state: Arc<Mutex<AppState>>) {
        let mut listen = Listener::new();

        executor::block_on(listen.connect_signal_loop(Arc::clone(&app_state)));

        let mut lyrics_fetcher = Genius::new();

        {
            let app_state = Arc::clone(&app_state);
            loop {
                listen.listen();

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

                    let (lyrics, cover_art) = match lyrics {
                        Ok(result) => (result.lyrics, Some(result.cover_art)),
                        _ => ("Lyrics not available".into(), None)
                    };

                    *app_state_guard = AppState::LyricsFetched {
                        song_name,
                        artist_name,
                        lyrics,
                        cover_art, 
                    };
                } else {
                    // no lyrics to be pulled, can sleep a bit
                    sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }

    pub fn update(&mut self, app_state: AppState) {
        match &app_state {
            AppState::LyricsFetched { lyrics, cover_art, .. } => {
                if self.app_state.fetched() {
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
            AppState::Connecting => (),
        }

        self.app_state = app_state;
    }
}
