use gio::SettingsExt;

pub struct Settings {
    settings: gio::Settings,
}

impl Settings {
    const SCHEMA_ID: &'static str = "com.github.onsah.sync-lyrics";
    const API_KEY_FIELD: &'static str = "api-key";

    pub fn new() -> Self {
        Settings {
            settings: gio::Settings::new(Self::SCHEMA_ID),
        }
    }

    pub fn set_api_key(&self, api_key: &str) {
        self.settings
            .set_string(Self::API_KEY_FIELD, api_key)
            .unwrap();
    }

    pub fn get_api_key(&self) -> String {
        // TODO: warn if fails to safe
        self.settings
            .get_string(Self::API_KEY_FIELD)
            .unwrap()
            .into()
    }
}
