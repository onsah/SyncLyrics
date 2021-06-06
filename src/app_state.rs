use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum AppState {
    Connecting,
    FetchingLyrics {
        song_name: String,
        artist_name: String,
    },
    LyricsFetched {
        song_name: String,
        artist_name: String,
        lyrics: String,
        cover_art: Option<Vec<u8>>,
    },
    SongNotFound {
        song_name: String,
        artist_name: String,
    },
    NetworkFailed,
}

impl AppState {
    pub fn fetched(&self) -> bool {
        match self {
            AppState::LyricsFetched { .. } => true,
            _ => false
        }
    }

    pub fn is_different(&self, new_song_name: &str, new_artist_name: &str) -> bool {
        match self {
            AppState::FetchingLyrics {
                song_name,
                artist_name,
            }
            | AppState::LyricsFetched {
                song_name,
                artist_name,
                ..
            } => song_name != new_song_name || artist_name != new_artist_name,
            _ => true
        }
    }
}

pub type AppStateWrapper = Arc<Mutex<AppState>>;
