//! Paint in-TypeChat links using Graphviz dot.

use std::{collections::HashSet, io};

use random_color::{Luminosity, RandomColor};

use crate::data::{Episode, Link};

/// Extract the number in a TypeChat URL
fn typechat_number(url: &str) -> Option<&str> {
    Some(
        url.strip_prefix("https://www.thetype.com/typechat/ep-")?
            .trim_end_matches("/"),
    )
}

/// Export episodes and links to a Graphviz dot file
pub fn paint(
    episodes: &Vec<Episode>,
    links: &Vec<Link>,
    mut buffer: impl io::Write,
) -> io::Result<()> {
    buffer.write(b"digraph {\nrankdir=LR\n")?;

    // Paint in-TypeChat links (edges) and record mentioned episodes
    let mut mentioned_url = HashSet::new();
    for l in links {
        if let Some(to_number) = typechat_number(&l.to_url) {
            let from_number =
                typechat_number(&l.from_url).expect("a link should start from a typechat URL");
            buffer.write(
                format!(
                    "typechat_{from_number} -> typechat_{to_number} [color=\"{color}\"]\n",
                    // To be more distinguishable
                    color = RandomColor::new().luminosity(Luminosity::Light).to_hex()
                )
                .as_bytes(),
            )?;

            mentioned_url.insert(&l.from_url);
            mentioned_url.insert(&l.to_url);
        }
    }

    // Paint mentioned episodes (nodes)
    for e in episodes {
        if mentioned_url.contains(&e.url) {
            buffer.write(
                format!(
                    "typechat_{number}[href=\"{href}\" label=\"{label}\"]\n",
                    label = e.name.replace("：", "\\n"),
                    href = e.url,
                    number = typechat_number(&e.url).expect("an episode's URL should be regular"),
                )
                .as_bytes(),
            )?;
        }
    }

    buffer.write(b"}")?;

    Ok(())
}
