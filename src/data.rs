//! Fetch data from the Internet.

use std::{
    collections::{hash_map::Entry, HashMap},
    thread, time,
};

use reqwest::blocking::get;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Episode {
    pub name: String,
    pub url: String,
}

impl PartialEq for Episode {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Hash for Episode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub from_url: String,
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
pub fn fetch_catalog() -> Vec<Episode> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortcutUrlCache(HashMap<String, String>);

impl ShortcutUrlCache {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Expand a shortcut URL (e.g. https://t.cn/zHVwH1H)
    fn expand(&mut self, url: &str) -> Result<String, reqwest::Error> {
        if url.starts_with("https://t.cn/") || url.starts_with("http://t.cn/") {
            println!("🔎 Expand “{}”.", url);

            match self.0.entry(url.to_owned()) {
                Entry::Occupied(e) => Ok(e.get().to_owned()),
                Entry::Vacant(e) => {
                    let response = get(e.key())?;

                    Ok(
                        e.insert(if let Some(location) = response.headers().get("location") {
                            location.to_str().unwrap().to_owned()
                        } else {
                            response.url().as_str().to_owned()
                        })
                        .to_owned(),
                    )
                }
            }
        } else {
            Ok(url.to_owned())
        }
    }
}

pub struct Driver {
    /// Short URL cache
    pub short_urls: ShortcutUrlCache,
    /// Links in episodes’ show notes
    pub episodes: HashMap<Episode, Vec<Link>>,
}

impl Driver {
    /// Fetch links in an episode’s show notes
    pub fn fetch_episode_detail(
        &mut self,
        episode: Episode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Entry::Vacant(ep) = self.episodes.entry(episode) {
            let links = Self::fetch_episode_detail_raw(ep.key(), &mut self.short_urls)
                .inspect_err(|err| eprintln!("failed to fetch episode detail: {err}."))?;
            ep.insert(links);
        }

        Ok(())
    }

    /// [`fetch_episode_detail`] without the episode cache
    fn fetch_episode_detail_raw(
        episode: &Episode,
        short_urls: &mut ShortcutUrlCache,
    ) -> Result<Vec<Link>, Box<dyn std::error::Error>> {
        println!("🚀 Fetching “{}”…", episode.name);

        let document = get(&episode.url)?.text()?;
        let document = Html::parse_document(&document);

        let selector = Selector::parse("#content > .typechat > .entry-content a")?;
        let links = document
            .select(&selector)
            .filter_map(|a| {
                if let Some(to_url) = a.value().attr("href") {
                    Some(to_url)
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
            .map(|to_url| {
                Ok(Link {
                    from_url: episode.url.to_owned(),
                    to_url: short_urls.expand(to_url)?.to_owned(),
                })
            })
            .collect::<Result<Vec<_>, reqwest::Error>>()?;

        println!("✅ Got {} links.", links.len());

        println!("💤 (Sleep for a second)");
        thread::sleep(time::Duration::from_secs(1));

        Ok(links)
    }
}
