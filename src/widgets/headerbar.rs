use gdk::prelude::{IsA};
use gtk::{
    Align, Image, ToggleButton,
    Widget, Label,
};
use gtk::prelude::{ButtonExt, GtkWindowExt, WidgetExt};

#[derive(Clone)]
pub struct HeaderBar {
    pub container: gtk::HeaderBar,
}

impl HeaderBar {
    pub fn new(window: impl GtkWindowExt) -> Self {
        let headerbar = gtk::HeaderBar::new();

        headerbar.set_title_widget(Some(&Label::new(Some("SyncLyrics"))));
        headerbar.set_show_title_buttons(true);

        headerbar.pack_start(&Self::create_pin_toggle(window));

        // headerbar.pack_end(&Self::create_switch(style_manager.clone()));
        headerbar.set_widget_name("headerbar");

        HeaderBar {
            container: headerbar,
        }
    }

    fn create_pin_toggle(_window: impl GtkWindowExt) -> impl IsA<Widget> {
        let toggle = ToggleButton::new();

        let toggle_image = Image::from_icon_name("view-pin-symbolic");
        
        toggle.set_child(Some(&toggle_image));

        toggle.set_tooltip_text(Some("Keep above"));
        toggle.set_valign(Align::Center);

        
        // TODO: Fix pin button
        toggle.set_visible(false);
        /* toggle.connect_toggled(move |f| {
            window.set_keeset_keep_abovep_above(f.get_active());
        }); */

        toggle
    }

    // TODO: Add color switch option in settings with 3 options:
    // 1. Default
    // 2. Dark
    // 3. Light
    /* fn create_switch(style_manager: adw::StyleManager) -> impl IsA<Widget> {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        hbox.set_margin_end(10);

        let dark_icon =
            Image::from_icon_name(Self::DARK_ICON_NAME);

        let light_icon =
            Image::from_icon_name(Self::LIGHT_ICON_NAME);

        let switch = Switch::new();

        switch.set_vexpand(false);
        switch.set_hexpand(false);
        switch.set_valign(Align::Center);

        switch.connect_state_set(move |_, enabled| {
            style_manager.set_color_scheme(match enabled {
                true => ColorScheme::PreferLight,
                false => ColorScheme::PreferDark,
            });
            Inhibit(false)
        });

        hbox.append(&dark_icon);
        hbox.append(&switch);
        hbox.append(&light_icon);

        hbox
    } */
}
