use glib::IsA;
use gtk::{Adjustment, ContainerExt, LabelExt, Widget, WidgetExt};

use crate::listener::SongInfo;

pub struct LyricsView {
    container: gtk::Box,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    lyrics_label: gtk::Label,
}

impl LyricsView {
    pub fn new() -> Self {
        let title_label = gtk::Label::new(Some(""));
        let artist_label = gtk::Label::new(Some(""));
        let lyrics_label = gtk::Label::new(Some(""));

        title_label.set_widget_name("title1");
        lyrics_label.set_line_wrap(true);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 5);
        container.add(&title_label);
        container.add(&artist_label);

        // Lyrics label is scrolled
        let label_scroller =
            gtk::ScrolledWindow::new(None as Option<&Adjustment>, None as Option<&Adjustment>);
        label_scroller.set_size_request(250, 250);
        label_scroller.set_vexpand(true);
        label_scroller.add(&lyrics_label);

        container.add(&label_scroller);

        container.show_all();

        LyricsView {
            container,
            title_label,
            artist_label,
            lyrics_label,
        }
    }

    pub fn as_widget(&self) -> &impl IsA<Widget> {
        &self.container
    }

    pub fn update(&mut self, song_info: &SongInfo) {
        self.set_song_title(&song_info.song_title);
        self.set_artist(&song_info.artist_name);
        self.set_lyrics(
            song_info
                .pull_lyrics
                .as_ref()
                .map(String::as_str)
                .unwrap_or("lyrics are not available"),
        );
    }

    fn set_song_title(&mut self, song_title: &str) {
        self.title_label
            .set_markup(&format!("<span size=\"x-large\">{}</span>", song_title));
    }

    fn set_artist(&mut self, artist_name: &str) {
        self.artist_label.set_markup(&format!(
            "<span size=\"large\" weight=\"bold\">{}</span>",
            artist_name
        ));
    }

    fn set_lyrics(&mut self, lyrics: &str) {
        self.lyrics_label
            .set_markup(&format!("<span size=\"large\">{}</span>", lyrics));
    }
}
