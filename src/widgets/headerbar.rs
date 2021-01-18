use glib::{IsA, ObjectExt};
use gtk::{Align, ButtonExt, ContainerExt, GtkWindowExt, HeaderBarExt, Image, Inhibit, Switch, SwitchExt, ToggleButton, ToggleButtonExt, Widget, WidgetExt};

pub struct HeaderBar {
    pub container: gtk::HeaderBar,
}

impl HeaderBar {

    const LIGHT_ICON_NAME: &'static str = "display-brightness-symbolic";
    const DARK_ICON_NAME: &'static str = "weather-clear-night-symbolic";

    pub fn new(window: impl GtkWindowExt) -> Self {
        let headerbar = gtk::HeaderBar::new();

        headerbar.set_title(Some("SyncLyrics"));
        headerbar.set_show_close_button(true);

        headerbar.pack_start(&Self::create_pin_toggle(window));

        headerbar.pack_end(&Self::create_switch());
        headerbar.set_widget_name("headerbar");

        HeaderBar {
            container: headerbar,
        }
    }   
    
    fn create_pin_toggle(window: impl GtkWindowExt) -> impl IsA<Widget> {
        let toggle = ToggleButton::new();

        toggle.set_image(Some(&Image::from_icon_name(Some("view-pin-symbolic"), gtk::IconSize::LargeToolbar)));
        toggle.set_tooltip_text(Some("Keep above"));
        toggle.set_valign(Align::Center);

        toggle.connect_toggled(move |f| {
            window.set_keep_above(f.get_active());
        });

        toggle
    }

    fn create_switch() -> impl IsA<Widget> {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);

        let dark_icon = Image::from_icon_name(Some(Self::DARK_ICON_NAME), gtk::IconSize::SmallToolbar);

        let light_icon = Image::from_icon_name(Some(Self::LIGHT_ICON_NAME), gtk::IconSize::SmallToolbar);

        let switch = Switch::new();

        switch.set_vexpand(false);
        switch.set_hexpand(false);
        switch.set_valign(Align::Center);

        let settings = gtk::Settings::get_default().unwrap();

        switch.connect_state_set(move |_, checked| {
            // TODO: change dark/light mode
            println!("checked: {}", checked);

            settings.set_property("gtk_application_prefer_dark_theme", &!checked).unwrap();

            Inhibit::default()
        });

        hbox.add(&dark_icon);
        hbox.add(&switch);
        hbox.add(&light_icon);
        
        hbox
    }
}