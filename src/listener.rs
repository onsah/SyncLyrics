use dbus::{Message, arg::{self, RefArg}};
use dbus::blocking::{Connection};
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

pub struct Listener {
    connection: Connection,
}

#[derive(Clone)]
pub struct SongInfo {
    pub song_title: String,
    pub artist_name: String,
    pub pull_lyrics: Option<String>,
}

struct PropertiesChanged {
    pub sender: String,
    pub title: Option<String>,
    pub artist: Option<String>,
}

impl Default for SongInfo {
    fn default() -> Self {
        SongInfo {
            song_title: "untitled".to_string(),
            artist_name: "unknown".to_string(),
            pull_lyrics: None,
        }
    }
}

impl arg::ReadAll for PropertiesChanged {
    fn read<'a>(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        let sender = i.read()?;
        let mut changed_properties: HashMap<String, arg::Variant<Box<dyn arg::RefArg>>> = i.read()?;
        
        let metadata = changed_properties.remove("Metadata").unwrap();
        let (title, artist) = get_title_and_artist(metadata);

        // TODO: read metadata from changed properties

        Ok(PropertiesChanged {
            sender,
            title,
            artist,
        })
    }
}

fn get_title_and_artist(metadata: arg::Variant<Box<dyn arg::RefArg>>) -> (Option<String>, Option<String>) {
    let (mut title, mut artist) = (None, None);

    let data = metadata.as_iter().unwrap().next().unwrap();
    let mut iter = data.as_iter().unwrap();

    while let Some(var) = iter.next().map(|v| v.box_clone()) {
        if let Some(s) = var.as_str() {
            // Tuple are flatten with the array
            match s {
                // next value of iterator is title
                "xesam:title" => if let Some(s) = iter.next().unwrap().as_str() {
                    title = Some(s.to_string());
                },
                // next value of iterator is artist
                "xesam:artist" => {
                    let artists = iter.next().unwrap();
                    let mut iter = artists.as_iter().unwrap();
                    let array = iter.next().unwrap();
                    let primary_artist = array.as_iter().unwrap().next().unwrap();
                    artist = Some(primary_artist.as_str().unwrap().to_string());
                },
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
            connection: Connection::new_session().expect("Couldn't create connection")
        }
    }

    pub fn connect_signal(&self, song_info: Arc<Mutex<SongInfo>>) {
        let proxy = self.connection.with_proxy("org.mpris.MediaPlayer2.spotify", "/org/mpris/MediaPlayer2", Duration::from_millis(5000));

        // TODO: retrieve current info
        use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
        let mut metadata: HashMap<String, arg::Variant<Box<dyn arg::RefArg>>> = proxy.get("org.mpris.MediaPlayer2.Player", "Metadata").unwrap();
        let title = metadata.remove("xesam:title").map(|s| s.as_str().unwrap().to_owned());
        // r/programminghorror
        let artist = metadata.remove("xesam:artist").map(|s| s.as_iter().unwrap()
            .next().unwrap()
            .as_iter().unwrap().next().unwrap()
            .as_str().unwrap().to_owned());
        match (title, artist) {
            (Some(song_title), Some(artist_name)) => {
                match song_info.try_lock() {
                    Ok(mut guard) => {
                        if &guard.song_title != &song_title {
                            *guard = SongInfo {
                                song_title,
                                artist_name,
                                pull_lyrics: None,
                            }
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        {
            let song_info = Arc::clone(&song_info);
            let _id = proxy.match_signal(move |p: PropertiesChanged, _: &Connection, _: &Message| {
                println!("{:#?} - {:#?}", p.title, p.artist);
                match (p.title, p.artist) {
                    (Some(song_title), Some(artist_name)) => {
                        match song_info.try_lock() {
                            Ok(mut guard) => {
                                if &guard.song_title != &song_title {
                                    println!("song changed to: {}", song_title);
                                    *guard = SongInfo {
                                        song_title,
                                        artist_name,
                                        pull_lyrics: None,
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
                true
            });
        }
    }

    pub fn listen(&mut self) {
        self.connection.process(Duration::from_millis(50)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listener_works() -> Result<(), Box<dyn std::error::Error>> {
        let song_info = Arc::from(Mutex::from(SongInfo::default()));
        let listener = Listener::new();
        
        let _ = listener.connect_signal(song_info);
        println!("connected");
        Ok(())
    }
}