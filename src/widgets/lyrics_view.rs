use glib::IsA;
use gtk::{Adjustment, ContainerExt, LabelExt, SpinnerExt, StackExt, Widget, WidgetExt};

pub struct LyricsView {
    container: gtk::Box,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    lyrics_label: gtk::Label,
    spinner: gtk::Spinner,
    stack: gtk::Stack,
}

impl LyricsView {
    pub fn new() -> Self {
        let title_label = gtk::Label::new(Some("This should change"));
        let artist_label = gtk::Label::new(Some("This should change"));
        let lyrics_label = gtk::Label::new(Some(""));

        title_label.set_widget_name("title1");
        lyrics_label.set_line_wrap(true);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 5);
        container.add(&title_label);
        container.add(&artist_label);

        let stack = gtk::Stack::new();

        let spinner = gtk::Spinner::new();
        spinner.set_size_request(75, 75);
        spinner.set_halign(gtk::Align::Center);
        spinner.set_valign(gtk::Align::Center);
        spinner.stop();

        stack.add_named(&spinner, "spinner");

        // Lyrics label is scrolled
        let label_scroller =
            gtk::ScrolledWindow::new(None as Option<&Adjustment>, None as Option<&Adjustment>);
        label_scroller.set_size_request(250, 250);
        label_scroller.set_vexpand(true);
        label_scroller.add(&lyrics_label);

        stack.add_named(&label_scroller, "lyrics");

        stack.set_visible_child_name("spinner");

        container.add(&stack);

        container.show_all();

        LyricsView {
            container,
            title_label,
            artist_label,
            lyrics_label,
            spinner,
            stack,
        }
    }

    pub fn as_widget(&self) -> &impl IsA<Widget> {
        &self.container
    }

    pub fn song_changed(&mut self, song_title: &str, artist_name: &str) {
        self.set_song_title(song_title);
        self.set_artist(artist_name);

        self.spinner.start();
        self.stack.set_visible_child_name("spinner");
    }

    fn set_song_title(&mut self, song_title: &str) {
        self.title_label.set_markup(&format!(
            "<span size=\"x-large\">{}</span>",
            Self::escape_markup(song_title)
        ));
    }

    fn set_artist(&mut self, artist_name: &str) {
        self.artist_label.set_markup(&format!(
            "<span size=\"large\" weight=\"bold\">{}</span>",
            Self::escape_markup(artist_name)
        ));
    }

    pub fn set_lyrics(&mut self, lyrics: &str) {
        self.lyrics_label.set_markup(&format!(
            "<span size=\"large\">{}</span>",
            Self::escape_markup(lyrics)
        ));
        self.spinner.stop();
        self.stack.set_visible_child_name("lyrics");
    }

    fn escape_markup(text: &str) -> String {
        text.replace("&", "&amp;")
    }
}
