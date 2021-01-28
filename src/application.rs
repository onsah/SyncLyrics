use futures::{
    executor,
    future::{AbortHandle, Abortable},
    Future,
};
use glib::Continue;
use gtk::{ApplicationWindow, ContainerExt, GtkWindowExt, Inhibit, WidgetExt};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

use crate::{
    listener::{Listener, SongInfo},
    lyrics::genius::Genius,
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

        self.song_info_start_listening(Arc::clone(&song_info));

        self.start_update_listener(song_info);
    }

    /**
     * Checks and updates if detected song is changed
     * Can't make async because gtk widgets are not Send
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
                Err(_) => (/* println!("update_listener: {:?}", e) */),
            }

            Continue(true)
        });
    }

    /**
     * Listens currently played song. If it changes it retrieves its lyrics as well
     */
    fn song_info_start_listening(&self, song_info: Arc<Mutex<SongInfo>>) {
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
    async fn song_info_listener_loop(song_info: Arc<Mutex<SongInfo>>) {
        let mut listen = Listener::new();

        executor::block_on(listen.connect_signal_loop(Arc::clone(&song_info)));

        let mut lyrics_fetcher = Genius::new();

        {
            let song_info = Arc::clone(&song_info);
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
                                .get_lyrics(&song_info.song_title, &song_info.artist_name)
                                .await;

                            song_info.pull_lyrics = Some(
                                lyrics
                                    .map(|l| l.lyrics)
                                    .unwrap_or("Lyrics not available".into()),
                            );
                        }
                    }
                    Err(_) => (/* println!("song_info_listener: {:?}", e) */),
                }

                sleep(Duration::from_millis(250)).await;
            }
        }
    }

    pub fn update(&mut self, song_info: SongInfo) {
        self.song_info = song_info;
        self.lyrics_view.update(&self.song_info);
    }
}
