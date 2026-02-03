//! Fetch data from the Internet.
//!
//! [`Fetcher`] fetches episodes and show notes, and [`Driver`] collect them into links.

use std::{
    collections::{HashMap, hash_map::Entry},
    io, thread, time,
};

use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// A fetcher that fetches episodes and show notes from WordPress API lazily.
#[derive(Default)]
pub struct Fetcher {
    pages_fetcher: PagesFetcher,
    /// Total number of episodes.
    len: usize,
}

impl Fetcher {
    pub fn build() -> reqwest::Result<Self> {
        let pages_fetcher = PagesFetcher::default();
        let len = pages_fetcher.load_first_page()?.total_posts;
        Ok(Self { pages_fetcher, len })
    }
    /// Get total number of episodes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Iterate over episodes and their show notes.
    pub fn iter(&mut self) -> impl Iterator<Item = (Episode, String)> + '_ {
        self.pages_fetcher
            .by_ref()
            .flat_map(|page| page.ok()) // Ignore network errors
            .flat_map(|page| {
                page.posts.into_iter().map(|post| {
                    (
                        Episode {
                            name: post.post_title,
                            url: post.link,
                        },
                        post.post_content,
                    )
                })
            })
    }
}

/// A fetcher that fetches pages from WordPress API lazily.
struct PagesFetcher {
    client: Client,
    /// The next page number to fetch.
    next_page: Option<usize>,
}
/// A page of posts in WordPress API.
#[derive(Debug, Deserialize)]
struct WpPostsPage {
    posts: Vec<WpPost>,
    total_posts: usize,
    total_pages: usize,
    current_page: usize,
}
/// A post in WordPress API.
#[derive(Debug, Deserialize)]
struct WpPost {
    /// Example: `字谈字畅 250：增补修订传心意`
    ///
    /// `title: "#250：增补修订传心意"` is also available. We choose `post_title` because it looks better in the final graph.
    post_title: String,
    /// Example: `https://www.thetype.com/typechat/ep-250/`
    link: String,
    /// HTML content.
    ///
    /// `summary`, and `post_excerpt` are also available, but they are truncated.
    post_content: String,
}

impl PagesFetcher {
    fn load_page(&self, page: usize) -> reqwest::Result<WpPostsPage> {
        let url = format!("https://www.thetype.com/wp-json/wp/v2/get-typechat-posts?page={page}",);
        let page_content: WpPostsPage = self.client.get(&url).send()?.json()?;
        assert_eq!(page_content.current_page, page);

        Ok(page_content)
    }

    fn fetch_next_page(&mut self) -> reqwest::Result<Option<WpPostsPage>> {
        if let Some(page) = self.next_page {
            let page_content = self.load_page(page)?;

            let next_page = page + 1;
            self.next_page = if next_page <= page_content.total_pages {
                Some(next_page)
            } else {
                None
            };

            Ok(Some(page_content))
        } else {
            Ok(None)
        }
    }

    fn load_first_page(&self) -> reqwest::Result<WpPostsPage> {
        self.load_page(1)
    }
}

impl Default for PagesFetcher {
    fn default() -> Self {
        Self {
            client: Client::new(),
            next_page: Some(1), // It starts from 1, not 0.
        }
    }
}

impl Iterator for PagesFetcher {
    type Item = reqwest::Result<WpPostsPage>;

    fn next(&mut self) -> Option<Self::Item> {
        self.fetch_next_page().transpose()
    }
}

/// A driver that collects episode show notes into a map of links.
pub struct Driver {
    /// Links in episodes’ show notes
    pub episodes: HashMap<Episode, Vec<String>>,
    /// Short URL cache
    short_urls: ShortcutUrlCache,
    // HTTP client
    client: Client,
}

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
struct ShortcutUrlCache(HashMap<String, String>);

impl ShortcutUrlCache {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Expand a shortcut URL (e.g. https://t.cn/zHVwH1H)
    fn expand(&mut self, url: &str, client: &Client) -> Result<String, reqwest::Error> {
        if url.starts_with("https://t.cn/") || url.starts_with("http://t.cn/") {
            println!("🔎 Expand “{}”.", url);

            match self.0.entry(url.to_owned()) {
                Entry::Occupied(e) => Ok(e.get().to_owned()),
                Entry::Vacant(e) => {
                    let response = client.get(e.key()).send()?;

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

impl Default for Driver {
    fn default() -> Self {
        Self {
            episodes: HashMap::new(),
            short_urls: ShortcutUrlCache::new(),
            client: Client::new(),
        }
    }
}

impl Driver {
    /// Save links in an episode’s show notes
    pub fn push_episode(
        &mut self,
        episode: Episode,
        show_notes: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Entry::Vacant(ep) = self.episodes.entry(episode) {
            let links =
                Self::push_episode_raw(ep.key(), show_notes, &mut self.short_urls, &self.client)
                    .inspect_err(|err| {
                        eprintln!("failed to push episode “{}”: {err}.", ep.key().name)
                    })?;
            ep.insert(links);
        }

        Ok(())
    }

    /// [`push_episode`] without the episode cache
    fn push_episode_raw(
        episode: &Episode,
        show_notes: &str,
        short_urls: &mut ShortcutUrlCache,
        client: &Client,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        println!("🚀 Fetching “{}”…", episode.name);

        let document = Html::parse_document(show_notes);

        let selector = Selector::parse("a")?;
        let links = document
            .select(&selector)
            .filter_map(|a| {
                if let Some(url) = a.value().attr("href") {
                    Some(url)
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
            .map(|url| Ok(short_urls.expand(url, client)?.to_owned()))
            .collect::<Result<Vec<_>, reqwest::Error>>()?;

        println!("✅ Got {} links.", links.len());

        println!("💤 (Sleep for a second)");
        thread::sleep(time::Duration::from_secs(1));

        Ok(links)
    }

    /// Build a `Driver` from JSON caches.
    pub fn from_cache(episodes: Option<String>, short_urls: Option<String>) -> io::Result<Self> {
        let episodes: HashMap<Episode, Vec<String>> = if let Some(episodes) = episodes {
            let vec: Vec<(_, _)> = serde_json::from_str(&episodes)?;
            vec.into_iter().collect()
        } else {
            HashMap::new()
        };

        let short_urls: ShortcutUrlCache = if let Some(short_urls) = short_urls {
            serde_json::from_str(&short_urls)?
        } else {
            ShortcutUrlCache::new()
        };

        Ok(Driver {
            episodes,
            short_urls,
            ..Default::default()
        })
    }

    /// Dump the `Driver` as JSON caches.
    pub fn to_cache(&self) -> io::Result<(String, String)> {
        let episodes: Vec<(_, _)> = self.episodes.iter().collect();
        let episodes_json = serde_json::to_string(&episodes)?;
        let short_urls_json = serde_json::to_string(&self.short_urls)?;
        Ok((episodes_json, short_urls_json))
    }
}
