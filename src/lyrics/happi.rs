use restson::{RestClient, RestPath};
use serde_derive::{Deserialize, Serialize};

use super::{LyricsError, LyricsFetcher, LyricsResponse, LyricsResult};
// implementation of LyricsFetcher for happi.dev lyrics api

pub struct HappiLyrics {
    api_key: String,
}

impl HappiLyrics {
    const BASE_ENDPOINT: &'static str = "https://api.happi.dev";
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchResponseRest {
    result: Vec<SearchResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    track: String,
    id_track: u32,
    artist: String,
    id_artist: u32,
    album: String,
    id_album: u32,
    haslyrics: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct LyricsResponseRest {
    result: LyricsResponse,
}

impl RestPath<()> for SearchResponseRest {
    fn get_path(_: ()) -> Result<String, restson::Error> {
        Ok("v1/music".into())
    }
}

impl RestPath<SearchResponse> for LyricsResponseRest {
    fn get_path(par: SearchResponse) -> Result<String, restson::Error> {
        Ok(format!(
            "v1/music/artists/{}/albums/{}/tracks/{}/lyrics",
            par.id_artist, par.id_album, par.id_track,
        ))
    }
}

impl HappiLyrics {
    pub fn search(&self, query: &str) -> Vec<SearchResponse> {
        let mut client = RestClient::new(Self::BASE_ENDPOINT).unwrap();

        let query = vec![
            ("q", query),
            ("apikey", &self.api_key),
            ("limit", ""),
            ("type", ""),
        ];

        // TODO: no unwrap
        let data: SearchResponseRest = client.get_with((), query.as_slice()).unwrap();

        data.result
    }
}

impl LyricsFetcher for HappiLyrics {
    fn new(api_key: String) -> Self {
        HappiLyrics { api_key }
    }

    fn get_lyrics(&self, song_title: &str, artist: &str) -> LyricsResult {
        let search_text = format!("{}, {}", song_title, artist);
        let results = self.search(&search_text);

        let found = results.into_iter().find(|r| r.haslyrics);

        match found {
            Some(search_resp) => {
                let mut client = RestClient::new(Self::BASE_ENDPOINT).unwrap();

                let query = vec![("apikey", self.api_key.as_str())];

                let LyricsResponseRest { result } =
                    client.get_with(search_resp, query.as_slice()).unwrap();

                Ok(result)
            }
            None => Err(LyricsError::NoLyrics),
        }
    }
}

#[cfg(test)]
mod tests {
    /* use super::HappiLyrics;
    use crate::lyrics::LyricsFetcher;

    const API_KEY: &'static str = "";
     You can uncomment these if you provide api key
    #[test]
    fn search_works() {
        let happi = HappiLyrics::new(API_KEY.into());

        let results = happi.search("Man in the Box Alice in Chains");

        assert_eq!(results[0].id_track, 721878);
    }

    #[test]
    fn lyrics_works() {
        let happi = HappiLyrics::new(API_KEY.into());

        let lyrics = happi.get_lyrics("Man in the Box", "Alice in Chains")
            .unwrap();

        lyrics.lyrics.split("\n").next().unwrap().contains("La, la, la, la, la, la, la, la, oh");
    } */
}
