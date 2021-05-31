use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use reqwest;
use std::fmt::Display;

pub mod genius;

pub trait LyricsFetcher {
    fn get_lyrics(&self, song_title: &str, artist: &str) -> LyricsResult;
}

pub type LyricsResult = Result<LyricsResponse, LyricsError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct LyricsResponse {
    pub artist: String,
    pub track: String,
    pub lyrics: String,
    pub cover_art: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum LyricsError {
    Network(#[from] reqwest::Error),
    SpotifyClosed,
    SongNotFound { song_name: String, artist: String },
}

impl Display for LyricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => f.write_fmt(format_args!("Network error: {}", e)),
            Self::SpotifyClosed => f.write_str("Spotify is not detected"),
            Self::SongNotFound { 
                song_name, artist
            } => f.write_fmt(format_args!("Song {} by {} is not found", song_name, artist))
        }
    }
}
