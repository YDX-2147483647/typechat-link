use std::{
    fs::{self, File},
    io::{Read, Write},
    thread, time,
};

mod data;

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
        let episodes = data::fetch_catalog();

        fs::create_dir_all(DATA_DIR)?;
        let mut file = File::create(EPISODES_DATA)?;
        file.write_all(serde_json::to_string(&episodes)?.as_bytes())?;

        episodes
    };

    // Load or fetch+save links
    let links = if let Ok(mut file) = File::open(LINKS_DATA) {
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
        let mut file = File::create(LINKS_DATA)?;
        file.write_all(serde_json::to_string(&links)?.as_bytes())?;
        links
    };

    println!("{} episodes and {} links.", episodes.len(), links.len());

    Ok(())
}
