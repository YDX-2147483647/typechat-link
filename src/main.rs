use std::{
    fs::{self, File},
    io::{Read, Write},
    thread, time,
};

use reqwest::blocking::get;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Episode {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
struct Link<'a> {
    pub from_url: &'a str,
    pub to_url: String,
}

impl Episode {
    fn from_anchor(anchor: ElementRef) -> Episode {
        Episode {
            name: anchor.inner_html(),
            url: anchor
                .value()
                .attr("href")
                .expect("an anchor should have a href")
                .to_owned(),
        }
    }
}

/// Fetch the catalog of episodes
fn fetch_catalog() -> Vec<Episode> {
    let document = get("https://www.thetype.com/typechat/")
        .unwrap()
        .text()
        .unwrap();
    let document = Html::parse_document(&document);

    let selector = Selector::parse("#post-9671 > .entry-content > ul:last-child > li > a").unwrap();
    document
        .select(&selector)
        .map(Episode::from_anchor)
        .collect()
}

/// Fetch links in an episode's show notes
fn fetch_episode_detail(episode: &Episode) -> Vec<Link> {
    let document = get(&episode.url).unwrap().text().unwrap();
    let document = Html::parse_document(&document);

    let selector = Selector::parse("#content > .typechat > .entry-content a").unwrap();
    document
        .select(&selector)
        .filter_map(|a| {
            if let Some(to_url) = a.value().attr("href") {
                Some(Link {
                    from_url: &episode.url,
                    to_url: to_url.to_owned(),
                })
            } else {
                // Example: Footer of https://www.thetype.com/typechat/ep-001/
                let html = &a.html();
                if html == "<a>｜</a>" || html == "<a></a>" {
                    None
                } else {
                    panic!("fail to get href from an anchor: {html}")
                }
            }
        })
        .collect()
}

const DATA_DIR: &str = "data";
const EPISODES_DATA: &str = "data/episodes.json";
const LINKS_DATA: &str = "data/links.json";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load or fetch+save episodes
    let episodes = if let Ok(mut file) = File::open(EPISODES_DATA) {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        serde_json::from_str(&buffer).unwrap()
    } else {
        let episodes = fetch_catalog();

        fs::create_dir_all(DATA_DIR)?;
        let mut file = File::create(EPISODES_DATA)?;
        file.write_all(serde_json::to_string(&episodes)?.as_bytes())?;

        episodes
    };

    // Fetch links
    let mut links = Vec::new();
    for e in &episodes {
        println!("🚀 Fetching “{}”…", e.name);

        let mut l = fetch_episode_detail(e);
        println!("✅ Got {} links.", l.len());
        links.append(&mut l);

        println!("💤 (Sleep for a second)");
        thread::sleep(time::Duration::from_secs(1));
    }

    // Save links
    let mut file = File::create(LINKS_DATA)?;
    file.write_all(serde_json::to_string(&links)?.as_bytes())?;

    Ok(())
}
