use std::io::Cursor;

use gdk_pixbuf::{Pixbuf, PixbufLoader, PixbufLoaderExt};
use glib::IsA;
use gtk::{Adjustment, ContainerExt, Image, ImageExt, Justification, LabelExt, OrientableExt, OverlayExt, SpinnerExt, StackExt, StyleContextExt, Widget, WidgetExt, WrapMode};
use image::ImageOutputFormat;

pub struct LyricsView {
    container: gtk::Box,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    cover_image: gtk::Image,
    background_image: gtk::Image,
    lyrics_label: gtk::Label,
    spinner: gtk::Spinner,
    stack: gtk::Stack,
    song_not_found_subtitle_label: gtk::Label,
}

const NETWORK_ERROR_VIEW_NAME: &'static str = "network_error";
const SONG_NOT_FOUND_NAME: &'static str = "song_not_found";

impl LyricsView {

    const COVER_IMAGE_SIZE: i32 = 75;
    const NO_COVER_ICON_NAME: &'static str = "folder-music-symbolic";

    pub fn new() -> Self {
        let top_overlay = gtk::Overlay::new();
        top_overlay.set_property_height_request(90);
        
        let background_image = gtk::Image::new();
        background_image.set_visible(false);
        background_image.set_opacity(0.6);

        let top_container = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        top_container.set_margin_start(15);
        top_container.set_margin_end(15);

        let cover_image = gtk::Image::new();
        cover_image.set_from_icon_name(Some(Self::NO_COVER_ICON_NAME), gtk::IconSize::Dialog);
        cover_image.set_size_request(Self::COVER_IMAGE_SIZE, Self::COVER_IMAGE_SIZE);
        top_container.add(&cover_image);

        let title_label = gtk::Label::new(Some(""));
        title_label.set_halign(gtk::Align::Start);
        title_label.set_margin_start(15);
        
        let artist_label = gtk::Label::new(Some(""));
        artist_label.set_halign(gtk::Align::Start);
        artist_label.set_margin_start(15);
        
        let text_container = gtk::Box::new(gtk::Orientation::Vertical, 5);
        text_container.set_margin_top(15);

        text_container.add(&title_label);
        text_container.add(&artist_label);

        top_container.add(&text_container);

        top_overlay.add_overlay(&background_image);
        top_overlay.add_overlay(&top_container);

        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
        let lyrics_label = gtk::Label::new(Some(""));

        lyrics_label.set_halign(gtk::Align::Start);
        lyrics_label.set_line_wrap(true);
        lyrics_label.set_margin_start(15);
        lyrics_label.set_margin_end(15);

        separator.set_hexpand(true);
        separator.set_margin_bottom(10);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_size_request(400, 500);
        container.set_hexpand(true);
        container.add(&top_overlay);
        container.add(&separator);

        let stack = gtk::Stack::new();

        // TODO: get welcome screen, etc.

        // Open spotify screen
        stack.add_named(&Self::get_not_connected_view(), "connecting");
        
        let network_error_view = Self::title_with_subtitle("Network error", "Check your internet connection");

        stack.add_named(&network_error_view, NETWORK_ERROR_VIEW_NAME);

        let song_not_found_subtitle_label = gtk::Label:: new(None);
        let song_not_found_view = Self::title_with_subtitle_from_labels(
            &gtk::Label::new(Some("Song Not Found")), 
            &song_not_found_subtitle_label.clone()
        );

        stack.add_named(&song_not_found_view, SONG_NOT_FOUND_NAME);
        
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

        container.add(&stack);

        container.show_all();

        LyricsView {
            container,
            title_label,
            artist_label,
            cover_image,
            background_image,
            lyrics_label,
            spinner,
            stack,
            song_not_found_subtitle_label,
        }
    }

    pub fn as_widget(&self) -> &impl IsA<Widget> {
        &self.container
    }

    pub fn song_changed(&mut self, song_title: &str, artist_name: &str) {
        self.set_song_title(song_title);
        self.set_artist(artist_name);
        self.cover_image.set_from_icon_name(Some(Self::NO_COVER_ICON_NAME), gtk::IconSize::Dialog);
        self.background_image.set_visible(false);

        self.spinner.start();
        self.stack.set_visible_child_name("spinner");
    }

    pub fn song_data_retrieved(&mut self, lyrics: &str, cover_art: Option<&[u8]>) {
        self.set_lyrics(lyrics);
        if let Some(cover_art) = cover_art {
            self.set_cover_art(cover_art);
        }
        self.spinner.stop();
        self.stack.set_visible_child_name("lyrics");
    }

    pub fn network_failed(&mut self) {
        self.spinner.stop();
        self.stack.set_visible_child_name(NETWORK_ERROR_VIEW_NAME);
    }

    pub fn song_not_found(&mut self, song_title: &str, artist_name: &str) {
        self.spinner.stop();
        self.song_not_found_subtitle_label.set_label(
            &format!("{} - {} could not be found", song_title, artist_name)  
        );
        self.stack.set_visible_child_name(SONG_NOT_FOUND_NAME);
    }

    fn get_not_connected_view() -> impl IsA<Widget> {
        Self::title_with_subtitle("Spotify is not detected", "You should launch Spotify")
    }

    fn title_with_subtitle(title: &str, subtitle: &str) -> impl IsA<Widget> {
        Self::title_with_subtitle_from_labels(
            &gtk::Label::new(Some(title)), 
            &gtk::Label::new(Some(subtitle))
        )
    }

    fn title_with_subtitle_from_labels(title_label: &gtk::Label, subtitle_label: &gtk::Label) -> impl IsA<Widget> {
        let title = title_label;
        title.set_justify(gtk::Justification::Center);
        title.set_hexpand(true);
        title.get_style_context().add_class("h1");

        let subtitle = subtitle_label;
        subtitle.set_justify(gtk::Justification::Center);
        subtitle.set_hexpand(true);
        subtitle.get_style_context().add_class("h2");
        subtitle.set_line_wrap(true);
        subtitle.set_max_width_chars(20);

        let content = gtk::Grid::new();
        content.set_hexpand(true);
        content.set_vexpand(true);
        content.set_orientation(gtk::Orientation::Vertical);
        content.set_valign(gtk::Align::Center);

        content.add(title);
        content.add(subtitle);

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
        /* self.spinner.stop();
        self.stack.set_visible_child_name("lyrics"); */
    }

    fn set_cover_art(&mut self, cover_art: &[u8]) {
        // Cover image
        /* let loader = gdk_pixbuf::PixbufLoader::new();
        loader.set_size(Self::COVER_IMAGE_SIZE, Self::COVER_IMAGE_SIZE);
        loader.write(cover_art).unwrap();
        loader.close().unwrap();

        if let Some(pixbuf) = loader.get_pixbuf() {
            self.cover_image.set_from_pixbuf(Some(&pixbuf));
        } */
        self.cover_image.set_from_pixbuf(Some(&Self::raw_to_pixbuf(cover_art, Self::COVER_IMAGE_SIZE, Self::COVER_IMAGE_SIZE)));

        // Background
        let img = image::load_from_memory(cover_art).unwrap()
            .thumbnail(300, 300);
            // .crop(0, 150 - 45, 300, 110);

        let img = img.blur(4.0);

        let mut buffer = Vec::new();
        img.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();

        let pixbuf = Self::raw_to_pixbuf(&buffer, 450, 450)
            .new_subpixbuf(0, 180, 450, 110).unwrap();

        self.background_image.set_from_pixbuf(Some(&pixbuf));
        self.background_image.set_visible(true);
    }

    fn raw_to_pixbuf(buffer: &[u8], width: i32, height: i32) -> Pixbuf {
        let loader = gdk_pixbuf::PixbufLoader::new();
        loader.set_size(width, height);
        loader.write(buffer).unwrap();
        loader.close().unwrap();

        loader.get_pixbuf().unwrap()
    }

    fn escape_markup(text: &str) -> String {
        text.replace("&", "&amp;")
    }
}
