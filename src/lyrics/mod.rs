use serde_derive::{Deserialize, Serialize};

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
    pub copyright_notice: String,
}

#[derive(Debug)]
pub enum LyricsError {
    /** Song does not have lyrics */
    NoLyrics,
}
