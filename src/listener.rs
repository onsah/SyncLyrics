use dbus::blocking::{stdintf::org_freedesktop_dbus::Properties, Connection};
use dbus::{
    arg::{self, RefArg},
    Message,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Mutex, time::sleep};

use crate::app_state::{AppState, AppStateWrapper};

pub struct Listener {
    connection: Connection,
}

struct PropertiesChanged {
    pub sender: String,
    pub title: Option<String>,
    pub artist: Option<String>,
}

/* impl Default for SongInfo {
    fn default() -> Self {
        SongInfo {
            song_title: "untitled".to_string(),
            artist_name: "unknown".to_string(),
            pull_lyrics: None,
        }
    }
} */

impl arg::ReadAll for PropertiesChanged {
    fn read<'a>(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        let sender = i.read()?;
        let mut changed_properties: HashMap<String, arg::Variant<Box<dyn arg::RefArg>>> =
            i.read()?;

        let metadata = changed_properties.remove("Metadata").unwrap();
        let (title, artist) = get_title_and_artist(metadata);

        Ok(PropertiesChanged {
            sender,
            title,
            artist,
        })
    }
}

fn get_title_and_artist(
    metadata: arg::Variant<Box<dyn arg::RefArg>>,
) -> (Option<String>, Option<String>) {
    let (mut title, mut artist) = (None, None);

    let data = metadata.as_iter().unwrap().next().unwrap();
    let mut iter = data.as_iter().unwrap();

    while let Some(var) = iter.next().map(|v| v.box_clone()) {
        if let Some(s) = var.as_str() {
            // Tuple are flatten with the array
            match s {
                // next value of iterator is title
                "xesam:title" => {
                    if let Some(s) = iter.next().unwrap().as_str() {
                        title = Some(s.to_string());
                    }
                }
                // next value of iterator is artist
                "xesam:artist" => {
                    let artists = iter.next().unwrap();
                    let mut iter = artists.as_iter().unwrap();
                    let array = iter.next().unwrap();
                    let primary_artist = array.as_iter().unwrap().next().unwrap();
                    artist = Some(primary_artist.as_str().unwrap().to_string());
                }
                _ => (),
            }
        }
    }
    (title, artist)
}

impl dbus::message::SignalArgs for PropertiesChanged {
    const NAME: &'static str = "PropertiesChanged";
    const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
}

impl Listener {
    pub fn new() -> Self {
        Listener {
            connection: Connection::new_session().expect("Couldn't create connection"),
        }
    }

    pub async fn connect_signal_loop(&self, app_state: AppStateWrapper) {
        while !self.connect_signal(Arc::clone(&app_state)).await.is_some() {
            sleep(Duration::from_millis(250)).await;
        }
    }

    async fn connect_signal(&self, app_state: AppStateWrapper) -> Option<()> {
        let proxy = self.connection.with_proxy(
            "org.mpris.MediaPlayer2.spotify",
            "/org/mpris/MediaPlayer2",
            Duration::from_millis(5000),
        );

        let metadata_res: Result<HashMap<String, arg::Variant<Box<dyn arg::RefArg>>>, _> =
            proxy.get("org.mpris.MediaPlayer2.Player", "Metadata");

        match metadata_res {
            Ok(mut metadata) => {
                let title: Option<String> = metadata
                    .remove("xesam:title")
                    .and_then(|s| Some(s.as_str()?.to_owned()));
                // r/programminghorror
                let artist: Option<String> = metadata.remove("xesam:artist").and_then(|s| {
                    Some(s.as_iter()?.next()?.as_iter()?.next()?.as_str()?.to_owned())
                });
                match (title, artist) {
                    (Some(song_title), Some(artist_name)) => match app_state.try_lock() {
                        Ok(mut guard) => {
                            // Sometimes empty song is passed which causes crash
                            Self::set_if_valid(&mut guard, &song_title, &artist_name);
                        }
                        _ => (),
                    },
                    _ => (),
                }

                {
                    let app_state = Arc::clone(&app_state);
                    let _id = proxy.match_signal(
                        move |p: PropertiesChanged, _: &Connection, _: &Message| {
                            println!("{:#?} - {:#?}", p.title, p.artist);
                            match (p.title, p.artist) {
                                (Some(song_title), Some(artist_name)) => match app_state.try_lock()
                                {
                                    Ok(mut guard) => {
                                        Self::set_if_valid(&mut guard, &song_title, &artist_name);
                                    }
                                    _ => (),
                                },
                                _ => (),
                            }
                            true
                        },
                    );
                }

                Some(())
            }
            _ => None,
        }
    }

    fn set_if_valid(app_state: &mut AppState, new_song_name: &str, new_artist_name: &str) {
        if new_song_name.is_empty() {

        } else {
            let should_change = app_state.is_different(new_song_name, new_artist_name);

            if should_change {
                println!("song changed to: {}", new_song_name);
                *app_state = AppState::FetchingLyrics {
                    song_name: new_song_name.to_string(),
                    artist_name: new_artist_name.to_string(),
                };
            }
        }
    }

    pub fn listen(&mut self) {
        self.connection.process(Duration::from_millis(50)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use futures::executor;

    use super::*;

    #[test]
    fn listener_works() -> Result<(), Box<dyn std::error::Error>> {
        executor::block_on(async {
            let song_info = Arc::from(Mutex::from(AppState::Connecting));
            let listener = Listener::new();

            assert!(listener.connect_signal(song_info).await.is_some());
            Ok(())
        })
    }
}
