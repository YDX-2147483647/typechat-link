use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    sync::{
        Arc,
        atomic::{self, Ordering::SeqCst},
    },
};

use data::{Driver, Episode};

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

fn load_driver() -> io::Result<Driver> {
    // Load episodes
    let episodes = fs::read_to_string(EPISODES_DATA)
        .inspect(|_| {
            println!("Loading episodes from {EPISODES_DATA}…");
        })
        .ok();

    // Load short URL cache
    let short_urls = fs::read_to_string(SHORT_URLS_DATA)
        .inspect(|_| {
            println!("Loading short URL cache from {SHORT_URLS_DATA}…");
        })
        .ok();

    Driver::from_cache(episodes, short_urls)
}

fn save_driver(driver: &Driver) -> io::Result<()> {
    let (episodes, short_urls) = &driver.to_cache()?;

    fs::create_dir_all(DATA_DIR)?;

    fs::write(EPISODES_DATA, episodes)?;
    fs::write(SHORT_URLS_DATA, short_urls)?;

    Ok(())
}

fn fetch_data() -> Result<Driver, Box<dyn std::error::Error>> {
    let mut driver = load_driver()?;

    let mut fetcher = data::Fetcher::build()?;
    println!("✅ Found {} episodes.", fetcher.len());

    // If Ctrl+C, stop updating episodes and [`save_driver`].
    let running = Arc::new(atomic::AtomicBool::new(true));
    let r = Arc::clone(&running);
    ctrlc::set_handler(move || {
        r.store(false, SeqCst);
    })
    .expect("error setting Ctrl-C handler");

    // Update episodes while `running`
    for (episode, show_notes) in fetcher.iter() {
        if running.load(SeqCst) {
            driver.push_episode(episode, &show_notes).inspect_err(|_| {
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

fn save_stats(episodes: &HashMap<Episode, Vec<String>>) -> io::Result<()> {
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

fn save_paint(episodes: HashMap<Episode, Vec<String>>) -> io::Result<()> {
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
