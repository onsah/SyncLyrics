use reqwest::Client;
use scraper::{Html, Selector};
use serde_derive::{Deserialize, Serialize};

use super::{LyricsResponse, LyricsResult};

static BASE_ENDPOINT: &'static str = "https://api.genius.com/";
static ACCESS_TOKEN: &'static str = env!("ACCESS_TOKEN");

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
    full_title: String,
    title: String,
    url: String,
}

pub struct Genius {
    client: Client,
}

impl Genius {
    pub fn new() -> Self {
        Genius {
            client: Client::new(),
        }
    }

    pub async fn get_lyrics(&mut self, song_title: &str, artist: &str) -> LyricsResult {
        let song_url = self.find_song_url(song_title, artist).await;

        // Get the page html
        let html = self
            .client
            .get(&song_url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let lyrics = Genius::extract_lyrics(html);

        Ok(LyricsResponse {
            track: song_title.into(),
            artist: artist.into(),
            lyrics: lyrics,
            copyright_notice: "".into(),
        })
    }

    async fn find_song_url(&mut self, song_title: &str, artist: &str) -> String {
        let url = BASE_ENDPOINT.to_string() + "search";
        let query: [(&str, &str); 1] = [("q", &(song_title.to_owned() + " " + artist))];

        let data: SearchResponse = self
            .client
            .get(&url)
            .query(&query)
            .header("Authorization", "Bearer ".to_string() + ACCESS_TOKEN)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        data.response.hits[0].result.url.clone()
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
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::Genius;

    #[tokio::test]
    async fn find_songpage_works() {
        let mut genius = Genius::new();

        let found = genius.find_song_url("HUMBLE", "Kendrick Lamar").await;
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