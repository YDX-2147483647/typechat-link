use std::{
    fs::{self, File},
    io::{Read, Write},
    thread, time,
};

mod data;
mod paint;

const DATA_DIR: &str = "data";
const EPISODES_DATA: &str = "data/episodes.json";
const LINKS_DATA: &str = "data/links.json";
const OUT_DIR: &str = "out";
const OUT_FILE: &str = "out/typechat.dot";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load or fetch+save episodes
    let episodes = if let Ok(mut file) = File::open(EPISODES_DATA) {
        println!("Loading episodes from {EPISODES_DATA}…");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        serde_json::from_str(&buffer).unwrap()
    } else {
        println!("Fetching episodes…");
        let episodes = data::fetch_catalog();

        println!("Saving episodes to {EPISODES_DATA}…");
        fs::create_dir_all(DATA_DIR)?;
        let mut file = File::create(EPISODES_DATA)?;
        file.write_all(serde_json::to_string(&episodes)?.as_bytes())?;

        episodes
    };

    // Load or fetch+save links
    let links = if let Ok(mut file) = File::open(LINKS_DATA) {
        println!("Loading links from {LINKS_DATA}…");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        serde_json::from_str(&buffer).unwrap()
    } else {
        let mut links = Vec::new();
        for e in &episodes {
            println!("🚀 Fetching “{}”…", e.name);

            let mut l = data::fetch_episode_detail(e);
            println!("✅ Got {} links.", l.len());
            links.append(&mut l);

            println!("💤 (Sleep for a second)");
            thread::sleep(time::Duration::from_secs(1));
        }

        // Save links
        println!("Saving links to {LINKS_DATA}…");
        let mut file = File::create(LINKS_DATA)?;
        file.write_all(serde_json::to_string(&links)?.as_bytes())?;
        links
    };

    println!("Data:");
    println!("  {} episodes.", episodes.len());
    println!(
        "  {} links. ({} point to thetype.com, {} point to typechat)",
        episodes.len(),
        links
            .iter()
            .filter(|&l| l.to_url.starts_with("https://www.thetype.com/"))
            .count(),
        links
            .iter()
            .filter(|&l| l.to_url.starts_with("https://www.thetype.com/typechat/ep-"))
            .count(),
    );

    println!("Saving to {OUT_FILE}…");
    fs::create_dir_all(OUT_DIR)?;
    let file = File::create(OUT_FILE)?;
    paint::paint(&episodes, &links, file)?;

    Ok(())
}
