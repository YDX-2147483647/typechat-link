//! Calculate URL statistics.

use std::collections::HashMap;

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
            "adobe.ly" => "www.adobe.com",
            "events.bizzabo.com" => "atypi.org",
            "www.atypi.org" => "atypi.org",
            "atypi2018.dryfta.com" => "atypi.org",
            "atypi2020.dryfta.com" => "atypi.org",
            "2020.typographics.com" => "typographics.com",
            "2021.typographics.com" => "typographics.com",
            "2022.typographics.com" => "typographics.com",
            "2023.typographics.com" => "typographics.com",
            "tokyotypedirectorsclub.org" => "www.tdc.org",
            "www.amazon.cn" => "www.amazon.com",
            "www.microsoft.com" => "microsoft.com",
            "kernelpanic.fm" => "pan.icu",
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
        "pan.icu" => "内核恐慌",
        "unicode.org" => "Unicode",
        "www.w3.org" => "W3C",
        "www.gb688.cn" => "国标",
        "en.wikipedia.org" => "Wikipedia (en)",
        "zh.wikipedia.org" => "Wikipedia (zh)",
        "ja.wikipedia.org" => "Wikipedia (ja)",
        "atypi.org" => "ATypI",
        "www.tdc.org" => "The Type Directors Club (TDC)",
        "www.granshan.com" => "GRANSHAN",
        "developer.mozilla.org" => "MDN Web Docs",
        "www.adobe.com" => "Adobe",
        "helpx.adobe.com" => "Adobe Help Center",
        "blogs.adobe.com" => "Adobe Blogs",
        "fonts.adobe.com" => "Adobe Fonts",
        "www.apple.com" => "Apple",
        "developer.apple.com" => "Apple Developer",
        "support.apple.com" => "Apple Support",
        "microsoft.com" => "Microsoft",
        "docs.microsoft.com" => "Microsoft Docs",
        "learn.microsoft.com" => "Microsoft Learn",
        "fonts.google.com" => "Google Fonts",
        "www.fontshop.com" => "FontShop",
        "www.foundertype.com" => "方正字库",
        "www.hanyi.com.cn" => "汉仪字库",
        "3type.cn" => "3type（三言）",
        "www.monotype.com" => "Monotype",
        "klim.co.nz" => "Klim Type Foundry",
        "www.myfonts.com" => "MyFonts (by Monotype)",
        "www.oneclub.org" => "The One Club",
        "glyphsapp.com" => "Glyphs",
        "www.fonts.com" => "Fonts.com",
        "typeproject.com" => "タイププロジェクト (Type Project)",
        "codepoints.net" => "Codepoints",
        "www.zdic.net" => "漢典",
        "github.com" => "GitHub",
        "baike.baidu.com" => "百度百科",
        "www.douban.com" => "豆瓣",
        "book.douban.com" => "豆瓣读书",
        "www.zhihu.com" => "知乎",
        "mp.weixin.qq.com" => "微信",
        "weibo.com" => "微博",
        "www.youtube.com" => "YouTube",
        "twitter.com" => "Twitter",
        "www.facebook.com" => "Facebook",
        "www.instagram.com" => "Instagram",
        "medium.com" => "Medium",
        "www.amazon.com" => "Amazon",
        "www.gov.cn" => "中央人民政府",
        "shanghaitype.org" => "上海活字",
        _ => url,
    }
}

/// Count links' references
pub fn count<'a>(links: impl Iterator<Item = &'a String>) -> HashMap<&'a str, i32> {
    let mut stats = HashMap::new();

    for l in links {
        if let Some(domain) = normalize(l) {
            stats
                .entry(domain)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    stats
}
