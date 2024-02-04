use reqwest::blocking::get;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    pub name: String,
    pub url: String,
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

/// Fetch links in an episode's show notes
pub fn fetch_episode_detail(episode: &Episode) -> Vec<Link> {
    let document = get(&episode.url).unwrap().text().unwrap();
    let document = Html::parse_document(&document);

    let selector = Selector::parse("#content > .typechat > .entry-content a").unwrap();
    document
        .select(&selector)
        .filter_map(|a| {
            if let Some(to_url) = a.value().attr("href") {
                Some(Link {
                    from_url: episode.url.clone(),
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
