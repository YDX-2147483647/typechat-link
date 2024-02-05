use std::io;

use crate::data::{Episode, Link};

/// Extract the number in a typechat URL
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
    buffer.write(b"strict digraph {\nrankdir=\"LR\"\n")?;

    for e in episodes {
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

    for l in links {
        let from_number =
            typechat_number(&l.from_url).expect("a link should start from a typechat URL");

        if let Some(to_number) = typechat_number(&l.to_url) {
            buffer.write(format!("typechat_{from_number} -> typechat_{to_number}\n").as_bytes())?;
        }
    }

    buffer.write(b"}")?;

    Ok(())
}
