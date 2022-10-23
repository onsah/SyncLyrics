use std::{time::Duration, collections::HashMap, thread::spawn};
use crossbeam_channel::Sender;

use dbus::{blocking::{Connection, stdintf::org_freedesktop_dbus::Properties}, arg::{self, RefArg}, Message};

use super::spotify_event::SpotifyEvent;

pub struct SpotifyEventProducer;

impl SpotifyEventProducer {
    pub fn init(sender: Sender<SpotifyEvent>) {
        let connection = Connection::new_session().expect("Couldn't create connection");
        spawn(move || {
            Self::init_spotify_listener(sender, &connection);

            loop {
                connection.process(Duration::from_millis(50)).unwrap();
            }
        });
    }

    fn init_spotify_listener(
        sender: Sender<SpotifyEvent>, 
        connection: &Connection
    ) {
        let proxy = connection.with_proxy(
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
                    (Some(song_title), Some(artist_name)) => 
                        match sender.send(SpotifyEvent::SongChanged { 
                            song_name: song_title, 
                            artist_name 
                        }) {
                            Ok(_) => (),
                            Err(error) => println!("{:?}", error),
                        } ,
                    _ => (),
                }

                {
                    let sender = sender.clone();
                    let _id = proxy.match_signal(
                        move |p: MaybePropertiesChanged, _: &Connection, _: &Message| {
                            match p.0 {
                                Some(PropertiesChanged { title, artist, .. }) =>
                                    {
                                        println!("spotify_event_producer: {:#?} - {:#?}", title, artist);
                                        match (title, artist) {
                                            (Some(song_title), Some(artist_name)) =>
                                                sender.send(SpotifyEvent::SongChanged { 
                                                    song_name: song_title, 
                                                    artist_name 
                                                }).unwrap(),
                                            _ => (),
                                        }
                                    },
                                None => println!("spotify_event_producer: No song changed"),
                            }
                            
                            true
                        },
                    );
                }

                ()
            }
            _ => (),
        }    
    }
}

struct PropertiesChanged {
    #[allow(dead_code)]
    pub sender: String,
    pub title: Option<String>,
    pub artist: Option<String>,
}

struct MaybePropertiesChanged(Option<PropertiesChanged>);

impl MaybePropertiesChanged {
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
}

impl arg::ReadAll for MaybePropertiesChanged {
    fn read<'a>(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        let sender = i.read()?;
        let mut changed_properties: HashMap<String, arg::Variant<Box<dyn arg::RefArg>>> =
            i.read()?;

        Ok(changed_properties.remove("Metadata").map(|metadata| {
            let (title, artist) = Self::get_title_and_artist(metadata);
    
            match (title, artist) {
                (Some(title), Some(artist)) =>
                    MaybePropertiesChanged(Some(PropertiesChanged{
                        sender,
                        title: Some(title),
                        artist: Some(artist),
                    })),
                _ => MaybePropertiesChanged(None)
            }
        }).unwrap_or(MaybePropertiesChanged(None)))
    }
}

impl dbus::message::SignalArgs for MaybePropertiesChanged {
    const NAME: &'static str = "PropertiesChanged";
    const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
}
