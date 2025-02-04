use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Write},
    sync::{
        atomic::{self, Ordering::SeqCst},
        Arc,
    },
};

use data::{Driver, Episode, ShortcutUrlCache};

mod data;
mod paint;
mod stats;

const DATA_DIR: &str = "data";
const EPISODES_DATA: &str = "data/episodes.json";
const SHORT_URLS_DATA: &str = "data/short_urls.json";
const OUT_DIR: &str = "out";
const OUT_PAINT: &str = "out/typechat.dot";
const OUT_STATS: &str = "out/external-links.md";
const MIN_LINK_REF: i32 = 11;

fn load_driver() -> Result<Driver, io::Error> {
    // Load episodes
    let episodes = if let Ok(mut file) = File::open(EPISODES_DATA) {
        println!("Loading episodes from {EPISODES_DATA}…");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let episodes: Vec<(_, _)> = serde_json::from_str(&buffer).unwrap();
        episodes.into_iter().collect()
    } else {
        HashMap::new()
    };

    // Load short URL cache
    let short_urls = if let Ok(mut file) = File::open(SHORT_URLS_DATA) {
        println!("Loading short URL cache from {SHORT_URLS_DATA}…");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        serde_json::from_str(&buffer).unwrap()
    } else {
        ShortcutUrlCache::new()
    };

    Ok(Driver::new(episodes, short_urls))
}

fn save_driver(driver: &Driver) -> Result<(), io::Error> {
    fs::create_dir_all(DATA_DIR)?;

    let mut file = File::create(EPISODES_DATA)?;
    let episodes: &Vec<(_, _)> = &driver.episodes.iter().collect();
    file.write_all(serde_json::to_string(episodes)?.as_bytes())?;

    let mut file = File::create(SHORT_URLS_DATA)?;
    file.write_all(serde_json::to_string(&driver.short_urls)?.as_bytes())?;

    Ok(())
}

fn fetch_data() -> Result<Driver, Box<dyn std::error::Error>> {
    let mut driver = load_driver()?;

    let catalog = data::fetch_catalog();
    println!("✅ Found {} episodes.", catalog.len());

    // If Ctrl+C, stop updating episodes and [`save_driver`].
    let running = Arc::new(atomic::AtomicBool::new(true));
    let r = Arc::clone(&running);
    ctrlc::set_handler(move || {
        r.store(false, SeqCst);
    })
    .expect("error setting Ctrl-C handler");

    // Update episodes while `running`
    for e in catalog {
        if running.load(SeqCst) {
            driver.fetch_episode_detail(e).inspect_err(|_| {
                // Save eagerly
                save_driver(&driver)
                    .inspect(|_| println!("cache saved after failure."))
                    .unwrap_or_else(|err| {
                        eprintln!("failed to save cache after failure: {err}");
                    })
            })?;
        } else {
            break;
        }
    }

    save_driver(&driver)?;

    if running.load(SeqCst) {
        Ok(driver)
    } else {
        panic!("shutdown by Ctrl+C.")
    }
}

fn save_stats(episodes: &HashMap<Episode, Vec<String>>) -> Result<(), io::Error> {
    println!("\nSaving to {OUT_STATS}…");
    let mut file = File::create(OUT_STATS)?;
    file.write_all(b"# Statistics of External Links\n\n")?;
    let unsorted_stats = stats::count(episodes.values().flatten());
    let mut sorted_stats: Vec<_> = unsorted_stats.iter().collect();
    sorted_stats.sort_unstable_by(|a, b| a.1.cmp(b.1).reverse());
    for (i, (domain, count)) in sorted_stats.iter().enumerate() {
        if **count >= MIN_LINK_REF {
            writeln!(
                file,
                "{i:02}. {:>3} [{}](https://{})",
                **count,
                stats::humanize(domain),
                domain
            )?;
        }
    }
    writeln!(
        file,
        "\nLinks with less than {} references are omitted.",
        MIN_LINK_REF
    )?;

    Ok(())
}

fn save_paint(episodes: HashMap<Episode, Vec<String>>) -> Result<(), io::Error> {
    let mut catalog: Vec<_> = episodes.keys().cloned().collect();
    // Sort to `paint` better
    catalog.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let links: Vec<_> = episodes
        .into_iter()
        .flat_map(|(ep, links)| {
            links
                .into_iter()
                .map(|to_url| paint::Link {
                    from_url: ep.url.to_owned(),
                    to_url,
                })
                .collect::<Vec<_>>()
        })
        .collect();

    println!("\nSaving to {OUT_PAINT}…");
    let file = File::create(OUT_PAINT)?;
    paint::paint(&catalog, &links, file)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = fetch_data()?;
    println!(
        "\n✅ Found {} links.",
        driver
            .episodes
            .values()
            .map(|links| links.len())
            .sum::<usize>()
    );

    fs::create_dir_all(OUT_DIR)?;
    save_stats(&driver.episodes)?;
    save_paint(driver.episodes)?;

    Ok(())
}
