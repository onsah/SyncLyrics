#[derive(Debug, Clone)]
pub enum SpotifyEvent {
    SongChanged {
        song_name: String,
        artist_name: String,
    }
}