//! Calculate URL statistics.

use std::collections::HashMap;

use crate::data::Link;

/// Simplify an external URL
///
/// Return `Some(url_origin)` for pertinent URLs, return `None` otherwise.
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
        // Extract the URL's origin
        let domain_start = url.find("://").expect("URL should have a scheme") + "://".len();
        let origin = if let Some(origin_end) = url[domain_start..].find("/") {
            &url[..domain_start + origin_end]
        } else {
            // If there is no path, URL's origin is itself.
            url
        };

        // Aliases, remove `www.`, `http` → `https`, etc.
        let origin = match origin {
            "http://www.unicode.org" => "https://unicode.org",
            "http://unicode.org" => "https://unicode.org",
            "https://www.unicode.org" => "https://unicode.org",
            "https://youtu.be" => "https://www.youtube.com",
            "https://events.bizzabo.com" => "https://atypi.org",
            "http://www.atypi.org" => "https://atypi.org",
            "https://www.atypi.org" => "https://atypi.org",
            "https://www.microsoft.com" => "https://microsoft.com",
            "http://baike.baidu.com" => "https://baike.baidu.com",
            "http://mp.weixin.qq.com" => "https://mp.weixin.qq.com",
            "http://3type.cn" => "https://3type.cn",
            "http://book.douban.com" => "https://book.douban.com",
            "http://www.hanyi.com.cn" => "https://www.hanyi.com.cn",
            any => any,
        };

        Some(origin)
    }
}

/// Humanize a URL
pub fn humanize(url: &str) -> &str {
    match url {
        "https://www.thetype.com" => "The Type",
        "https://ipn.li" => "IPN",
        "https://unicode.org" => "Unicode",
        "https://www.w3.org" => "W3C",
        "https://en.wikipedia.org" => "Wikipedia (en)",
        "https://zh.wikipedia.org" => "Wikipedia (zh)",
        "https://ja.wikipedia.org" => "Wikipedia (ja)",
        "https://atypi.org" => "ATypI",
        "https://www.tdc.org" => "The Type Directors Club (TDC)",
        "https://www.granshan.com" => "GRANSHAN",
        "https://developer.mozilla.org" => "MDN Web Docs",
        "https://helpx.adobe.com" => "Adobe Help Center",
        "https://developer.apple.com" => "Apple Developer",
        "https://microsoft.com" => "Microsoft",
        "https://docs.microsoft.com" => "Microsoft Docs",
        "https://learn.microsoft.com" => "Microsoft Learn",
        "https://www.fontshop.com" => "FontShop",
        "https://www.foundertype.com" => "方正字库",
        "https://www.hanyi.com.cn" => "汉仪字库",
        "https://3type.cn" => "3type（三言）",
        "https://www.monotype.com" => "Monotype",
        "https://www.myfonts.com" => "MyFonts (by Monotype)",
        "https://www.oneclub.org" => "The One Club",
        "https://glyphsapp.com" => "Glyphs",
        "https://www.fonts.com" => "Fonts.com",
        "https://codepoints.net" => "Codepoints",
        "https://github.com" => "GitHub",
        "https://baike.baidu.com" => "百度百科",
        "https://book.douban.com" => "豆瓣读书",
        "https://www.zhihu.com" => "知乎",
        "https://mp.weixin.qq.com" => "微信",
        "https://www.youtube.com" => "YouTube",
        "https://twitter.com" => "Twitter",
        "https://www.amazon.com" => "Amazon",
        _ => url,
    }
}

/// Count links' references
pub fn count(links: &Vec<Link>) -> HashMap<&str, i32> {
    let mut stats = HashMap::new();

    for l in links {
        if let Some(url) = normalize(&l.to_url) {
            stats
                .entry(url)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    stats
}
