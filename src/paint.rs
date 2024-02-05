use std::{collections::HashMap, io};

use crate::data::{Episode, Link};

/// Extract the number in a TypeChat URL
fn typechat_number(url: &str) -> Option<&str> {
    Some(
        url.strip_prefix("https://www.thetype.com/typechat/ep-")?
            .trim_end_matches("/"),
    )
}

/// Simplify an external URL
///
/// Return `Some(url_origin)` for pertinent URLs, return `None` otherwise.
fn simplify_url(url: &str) -> Option<&str> {
    // Match constants
    let irrelevant = match url {
        // 各平台《字谈字畅》
        "https://www.thetype.com/typechat/feed/"
        | "http://www.lizhi.fm/1852153/"
        | "http://music.163.com/#/djradio?id=346541057"
        | "https://www.thetype.com/feed/typechat/"
        | "https://itunes.apple.com/cn/podcast/zi-tan-zi-chang/id1041704528"
        | "https://podcasts.apple.com/cn/podcast/%E5%AD%97%E8%B0%88%E5%AD%97%E7%95%85/id1041704528"
        | "https://itunes.apple.com/cn/podcast/%E5%AD%97%E8%B0%88%E5%AD%97%E7%95%85/id1041704528" 
        | "https://static.thetype.cloud/typechat/assets/typechat-weapp.jpg" => true,
        // 会员
        "https://www.thetype.com/members/" => true,
        // 特例
        // 写真歴史博物館——https://www.thetype.com/typechat/ep-039/
        "%E5%86%99%E7%9C%9F%E6%AD%B4%E5%8F%B2%E5%8D%9A%E7%89%A9%E9%A4%A8" => true,
        _ =>             false,
    };
    if irrelevant {
        return None;
    }

    // Cloudflare email protection, or a literal email
    if url.starts_with("/cdn-cgi/l/email-protection")
        || url.starts_with("mailto:")
        // Audio
        || (url.starts_with("https://static.thetype.cloud/typechat/") && url.ends_with(".mp3"))
    {
        None
    } else {
        let domain_start = url.find("://").expect("URL should have a scheme") + "://".len();
        if let Some(origin_end) = url[domain_start..].find("/") {
            Some(&url[..domain_start + origin_end])
        } else {
            // If there is no path, URL's origin is itself.
            Some(url)
        }
    }
}

/// Humanize a domain
fn humanize_domain(domain: &str) -> &str {
    match domain {
        "www.thetype.com" => "The Type",
        "en.wikipedia.org" => "Wikipedia (en)",
        "zh.wikipedia.org" => "Wikipedia (zh)",
        "ja.wikipedia.org" => "Wikipedia (ja)",
        "github.com" => "GitHub",
        "baike.baidu.com" => "百度百科",
        "www.w3.org" => "W3C",
        "book.douban.com" => "豆瓣读书",
        "unicode.org" => "Unicode",
        "zhihu.com" => "知乎",
        "microsoft.com" => "Microsoft",
        "ipn.li" => "IPN",
        "mp.weixin.qq.com" => "微信",
        "events.bizzabo.com" => "ATypI",
        _ => domain,
    }
}

/// Sanitize a URL as an ID in Graphviz dot
fn url_to_id(str: &str) -> String {
    str.replace("://", "__").replace(".", "_")
}

/// Export episodes and links to a Graphviz dot file
pub fn paint(
    episodes: &Vec<Episode>,
    links: &Vec<Link>,
    mut buffer: impl io::Write,
) -> io::Result<()> {
    buffer.write(b"digraph {\nrankdir=LR\n")?;

    // Paint episodes (nodes)
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

    let mut external_urls = HashMap::new();

    // Paint in-TypeChat links (edges) and count external URLs
    for l in links {
        if let Some(to_number) = typechat_number(&l.to_url) {
            let from_number =
                typechat_number(&l.from_url).expect("a link should start from a typechat URL");
            buffer.write(
                format!("typechat_{from_number} -> typechat_{to_number} [color=orange]\n")
                    .as_bytes(),
            )?;
        } else if let Some(url) = simplify_url(&l.to_url) {
            external_urls
                .entry(url)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    const MIN_COUNT: i32 = 50;

    // Paint external URLs (nodes)
    for (url, count) in &external_urls {
        if *count > MIN_COUNT {
            buffer.write(
                format!(
                    "external_{id}[href=\"{href}\" label=\"{label}\" shape=plaintext]\n",
                    label = humanize_domain(
                        url.trim_start_matches("https://")
                            .trim_start_matches("http://")
                    ),
                    href = url,
                    id = url_to_id(url),
                )
                .as_bytes(),
            )?;
        }
    }

    // Paint external links (edges)
    for l in links {
        if typechat_number(&l.to_url).is_none() {
            if let Some(to_url) = simplify_url(&l.to_url) {
                if *external_urls.get(to_url).unwrap_or(&0) > MIN_COUNT {
                    let from_number = typechat_number(&l.from_url)
                        .expect("a link should start from a typechat URL");
                    buffer.write(
                        format!(
                            "typechat_{from_number} -> external_{} [style=dashed color=gray]\n",
                            url_to_id(to_url)
                        )
                        .as_bytes(),
                    )?;
                }
            }
        }
    }

    buffer.write(b"}")?;

    Ok(())
}
