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
    },
}

impl AppState {
    pub fn fetched(&self) -> bool {
        match self {
            AppState::Connecting | AppState::FetchingLyrics { .. } => false,
            _ => true,
        }
    }

    pub fn is_different(&self, new_song_name: &str, new_artist_name: &str) -> bool {
        match self {
            AppState::Connecting => true,
            AppState::FetchingLyrics {
                song_name,
                artist_name,
            }
            | AppState::LyricsFetched {
                song_name,
                artist_name,
                ..
            } => song_name != new_song_name || artist_name != new_artist_name,
        }
    }
}

pub type AppStateWrapper = Arc<Mutex<AppState>>;
