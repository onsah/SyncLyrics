use glib::IsA;
use gtk::{
    Adjustment, ContainerExt, ImageExt, Justification, LabelExt, OrientableExt, SpinnerExt,
    StackExt, StyleContextExt, Widget, WidgetExt,
};

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
        let top_container = gtk::Box::new(gtk::Orientation::Horizontal, 5);

        let cover_image_view = gtk::Image::new();
        cover_image_view.set_from_icon_name(Some("folder-music-symbolic"), gtk::IconSize::Dialog);
        cover_image_view.set_size_request(50, 50);
        top_container.add(&cover_image_view);

        let title_label = gtk::Label::new(Some(""));
        let artist_label = gtk::Label::new(Some(""));
        let text_container = gtk::Box::new(gtk::Orientation::Vertical, 5);

        text_container.add(&title_label);
        text_container.add(&artist_label);

        top_container.add(&text_container);

        title_label.set_halign(gtk::Align::Start);
        title_label.set_margin_start(15);

        artist_label.set_halign(gtk::Align::Start);
        artist_label.set_margin_start(15);

        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
        let lyrics_label = gtk::Label::new(Some(""));

        lyrics_label.set_halign(gtk::Align::Start);
        lyrics_label.set_line_wrap(true);

        separator.set_hexpand(true);
        separator.set_margin_bottom(10);
        separator.set_margin_top(10);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_size_request(400, 500);
        container.set_margin_start(15);
        container.set_margin_end(15);
        container.add(&top_container);
        container.add(&separator);

        let stack = gtk::Stack::new();

        // TODO: get welcome screen, etc.

        // Open spotify screen
        stack.add_named(&Self::get_not_connected_view(), "connecting");
        // Fetching lyrics screen
        let spinner = gtk::Spinner::new();
        spinner.set_size_request(75, 75);
        spinner.set_halign(gtk::Align::Center);
        spinner.set_valign(gtk::Align::Center);
        spinner.stop();

        stack.add_named(&spinner, "spinner");

        // Lyrics label is scrolled
        let label_scroller =
            gtk::ScrolledWindow::new(None as Option<&Adjustment>, None as Option<&Adjustment>);
        label_scroller.set_vexpand(true);
        label_scroller.add(&lyrics_label);

        stack.add_named(&label_scroller, "lyrics");

        stack.set_visible_child_name("connecting");

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

    fn get_not_connected_view() -> impl IsA<Widget> {
        let title = gtk::Label::new(Some("Spotify is not detected"));
        title.set_justify(gtk::Justification::Center);
        title.set_hexpand(true);
        title.get_style_context().add_class("h1");

        let subtitle = gtk::Label::new(Some("You should launch Spotify"));
        subtitle.set_justify(gtk::Justification::Center);
        subtitle.set_hexpand(true);
        subtitle.get_style_context().add_class("h2");

        let content = gtk::Grid::new();
        content.set_hexpand(true);
        content.set_vexpand(true);
        content.set_orientation(gtk::Orientation::Vertical);
        content.set_valign(gtk::Align::Center);

        content.add(&title);
        content.add(&subtitle);

        content
    }

    fn set_song_title(&mut self, song_title: &str) {
        self.title_label.set_markup(&format!(
            "<span size=\"xx-large\">{}</span>",
            Self::escape_markup(song_title)
        ));
    }

    fn set_artist(&mut self, artist_name: &str) {
        self.artist_label.set_markup(&format!(
            "<span size=\"x-large\" weight=\"bold\">{}</span>",
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
