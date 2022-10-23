#[derive(Debug)]
pub enum UIEvent {
    SongInformationFetched {
        song_name: String,
        artist_name: String,
        lyrics: String,
        cover_art: Vec<u8>,
    },
    FetchingLyrics {
        song_name: String,
        artist_name: String,
    },
}