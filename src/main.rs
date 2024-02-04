use std::{
    fs::{self, File},
    io::Write,
};

use reqwest::blocking::get;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Episode {
    pub name: String,
    pub url: String,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let episodes = fetch_catalog();

    fs::create_dir_all("data")?;
    let mut file = File::create("data/episodes.json")?;
    file.write_all(serde_json::to_string(&episodes)?.as_bytes())?;

    Ok(())
}
