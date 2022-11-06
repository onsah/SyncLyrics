use std::{usize};

use futures::try_join;
use reqwest::{Client, ClientBuilder};
use scraper::{Html, Selector};
use serde_derive::{Deserialize, Serialize};

use crate::configs::NETWORK_TIMEOUT_DURATION;

use super::{LyricsError, LyricsResponse, LyricsResult};

static BASE_ENDPOINT: &'static str = "https://api.genius.com/";
static ACCESS_TOKEN: &'static str = env!("ACCESS_TOKEN");

#[derive(Serialize, Deserialize, Debug)]
struct SongResponseWrapper {
    response: SongResponse,
}

#[derive(Serialize, Deserialize, Debug)]
struct SongResponse {
    song: SongResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
struct SongResponseData {
    title: String,
    url: String,
    album: SongResponseAlbum,                                                             
}

#[derive(Serialize, Deserialize, Debug)]
struct SongResponseAlbum {
    cover_art_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchResponse {
    response: SearchResponseReal,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchResponseReal {
    hits: Vec<SearchResponseEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchResponseEntry {
    #[serde(rename = "type")]
    type_: String,
    result: SearchResponseResult,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchResponseResult {
    id: usize,
}

pub struct Genius {
    client: Client,
}

impl Genius {
    pub fn new() -> Self {
        Genius {
            client: ClientBuilder::new()
                .timeout(NETWORK_TIMEOUT_DURATION)
                .build()
                .unwrap(),
        }
    }

    pub async fn get_lyrics(&mut self, song_title: &str, artist: &str) -> LyricsResult {
        
        let song_info = {
            let song_id = self.request_song_id(song_title, artist).await?;
            self.request_song_info(song_id).await?
        };
        
        let lyrics_future = {
            let song_url = &song_info.url;
            let client = self.client.clone();
            
            async move {
                let html = client
                    .get(song_url)
                    .send()
                    .await?
                    .text()
                    .await?;
    
                println!("Lyrics fetched");

                let lyrics = Genius::extract_lyrics(html);  
    
                Ok(lyrics)
            }
        };

        let cover_art_future = async {
            let cover_art = self.get_cover_art(&song_info.album).await?;
            println!("Covert art fetched");
            Ok::<_, LyricsError>(cover_art)
        };
        
        let (cover_art, lyrics) = try_join!(
            cover_art_future,
            lyrics_future
        )?;

        Ok(LyricsResponse {
            track: song_title.into(),
            artist: artist.into(),
            lyrics: lyrics,
            cover_art,
        })
    }

    async fn request_song_info(&mut self, song_id: usize) -> LyricsResult<SongResponseData> {
        let url = BASE_ENDPOINT.to_string() + "songs/" + &song_id.to_string();

        let resp: SongResponseWrapper = self
            .client
            .get(&url)
            .header("Authorization", "Bearer ".to_string() + ACCESS_TOKEN)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.response.song)
    }

    async fn request_song_id(&mut self, song_title: &str, artist: &str) -> LyricsResult<usize> {
        let url = BASE_ENDPOINT.to_string() + "search";
        let query: [(&str, &str); 1] = [("q", &(song_title.to_owned() + " " + artist))];

        let resp: SearchResponse = self
            .client
            .get(&url)
            .query(&query)
            .header("Authorization", "Bearer ".to_string() + ACCESS_TOKEN)
            .send()
            .await?
            .json()
            .await?;

        match resp.response.hits.get(0) {
            Some(hit) => Ok(hit.result.id),
            None => Err(LyricsError::SongNotFound { 
                song_name: song_title.to_string(), 
                artist: artist.to_string() 
            }),
        }
    }

    async fn get_cover_art(&mut self, album: &SongResponseAlbum) -> LyricsResult<Vec<u8>> {
        // TODO convert this to 300x300 url
        let url = &album.cover_art_url;

        let resp = self
            .client
            .get(url)
            .send()
            .await?
            .bytes()
            .await?;

        Ok(resp.into_iter().collect())
    }

    fn extract_lyrics(html: String) -> String {
        let doc = Html::parse_document(&html);
        let selector1 = Selector::parse("div.lyrics").unwrap();
        let selector2 = Selector::parse("div[class^=\"Lyrics__Container\"]").unwrap();

        let lyrics_divs = doc.select(&selector1).chain(doc.select(&selector2));

        lyrics_divs
            .map(|d| {
                d.text()
                    .map(|s| if s.contains("\n") { s.trim() } else { s })
                    .filter(|&s| s.chars().any(|c| !c.is_whitespace()))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>()
            .join("")
            .replace("[", "\n[")
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Genius;

    #[tokio::test]
    async fn find_songpage_works() {
        let mut genius = Genius::new();

        let song_id = genius.request_song_id("HUMBLE", "Kendrick Lamar").await.unwrap();
        let song_info = genius.request_song_info(song_id).await.unwrap();
        let found = song_info.url;
        assert_eq!(&found, "https://genius.com/Kendrick-lamar-humble-lyrics");
    }

    #[test]
    fn extract_lyrics_works() {
        let url = "https://genius.com/Tool-fear-inoculum-lyrics";

        let answer = "[Verse 1]
Immunity, long overdue
Contagion, I exhale you
Naive, I opened up to you
Venom and mania
Now, contagion, I exhale you
[Interlude]
The deceiver says, he says
\"You belong to me
You don't wanna breathe the light of the others
Fear the light, fear the breath
Fear the others for eternity\"
But I hear them now, inhale the clarity
Hear the venom, the venom in
What you say inoculated
Bless this immunity
Bless this immunity
Bless this immunity
[Chorus]
Exhale, expel
Recast my tale
Weave my allegorical elegy
[Verse 2]
Enumerate all that I'm to do
Calculating steps away from you
My own mitosis
Growing through division from mania
[Chorus]
Exhale, expel
Recast my tale
Weave my allegorical elegy
[Bridge]
Forfeit all control
You poison, you spectacle
Exorcise the spectacle
Exorcise the malady
Exorcise the disparate
Poison for eternity
Purge me and evacuate
The venom and the fear that binds me
[Outro]
Your veil now, lift away
I see you runnin'
Deceiver chased away
A long time comin'"
            .split("\n")
            .map(|s| s.trim())
            .collect::<Vec<_>>();

        let html = reqwest::blocking::get(url).unwrap().text().unwrap();

        let lyrics = Genius::extract_lyrics(html);

        let lyrics = lyrics
            .split("\n")
            .filter(|&s| s.chars().any(|c| !c.is_whitespace()))
            .map(|s| s.trim())
            .collect::<Vec<_>>();

        assert_eq!(answer, lyrics);
    }
}
