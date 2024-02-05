//! Calculate URL statistics.

use std::collections::HashMap;

use crate::data::Link;

/// Simplify an external URL
///
/// Return `Some(domain)` for pertinent URLs, return `None` otherwise.
fn normalize(url: &str) -> Option<&str> {
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
        _ => false,
    };
    if irrelevant {
        return None;
    }

    // Cloudflare email protection, or a literal email
    if url.starts_with("/cdn-cgi/l/email-protection")
        || url.starts_with("mailto:")
        // Audio
        || (url.starts_with("https://static.thetype.cloud/typechat/") && url.ends_with(".mp3"))
        // Images and files
        || url.starts_with("https://static.thetype.cloud/typechat/assets/")
    {
        None
    } else {
        // Extract the URL's domain
        let domain = url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        let domain = domain.split("/").next().expect("URL contains a domain");

        // Aliases, remove `www.`, etc.
        let domain = match domain {
            "www.unicode.org" => "unicode.org",
            "youtu.be" => "www.youtube.com",
            "events.bizzabo.com" => "atypi.org",
            "www.atypi.org" => "atypi.org",
            "www.microsoft.com" => "microsoft.com",
            any => any,
        };

        Some(domain)
    }
}

/// Humanize a URL
pub fn humanize(url: &str) -> &str {
    match url {
        "www.thetype.com" => "The Type",
        "ipn.li" => "IPN",
        "unicode.org" => "Unicode",
        "www.w3.org" => "W3C",
        "en.wikipedia.org" => "Wikipedia (en)",
        "zh.wikipedia.org" => "Wikipedia (zh)",
        "ja.wikipedia.org" => "Wikipedia (ja)",
        "atypi.org" => "ATypI",
        "www.tdc.org" => "The Type Directors Club (TDC)",
        "www.granshan.com" => "GRANSHAN",
        "developer.mozilla.org" => "MDN Web Docs",
        "helpx.adobe.com" => "Adobe Help Center",
        "developer.apple.com" => "Apple Developer",
        "microsoft.com" => "Microsoft",
        "docs.microsoft.com" => "Microsoft Docs",
        "learn.microsoft.com" => "Microsoft Learn",
        "www.fontshop.com" => "FontShop",
        "www.foundertype.com" => "方正字库",
        "www.hanyi.com.cn" => "汉仪字库",
        "3type.cn" => "3type（三言）",
        "www.monotype.com" => "Monotype",
        "www.myfonts.com" => "MyFonts (by Monotype)",
        "www.oneclub.org" => "The One Club",
        "glyphsapp.com" => "Glyphs",
        "www.fonts.com" => "Fonts.com",
        "codepoints.net" => "Codepoints",
        "github.com" => "GitHub",
        "baike.baidu.com" => "百度百科",
        "book.douban.com" => "豆瓣读书",
        "www.zhihu.com" => "知乎",
        "mp.weixin.qq.com" => "微信",
        "www.youtube.com" => "YouTube",
        "twitter.com" => "Twitter",
        "www.amazon.com" => "Amazon",
        _ => url,
    }
}

/// Count links' references
pub fn count(links: &Vec<Link>) -> HashMap<&str, i32> {
    let mut stats = HashMap::new();

    for l in links {
        if let Some(domain) = normalize(&l.to_url) {
            stats
                .entry(domain)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    stats
}
