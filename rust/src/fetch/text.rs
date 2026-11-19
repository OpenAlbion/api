//! Cleaning and derivation helpers shared by the parsers.
//!
//! These replicate the `Str::` macros registered in
//! `app/Providers/WikiServiceProvider.php` plus the small string cleanups the
//! Laravel actions perform.

/// Trim surrounding whitespace (the equivalent of Symfony DomCrawler's
/// `text()`, which trims but does not collapse inner whitespace).
pub fn text_trim(s: &str) -> String {
    s.trim().to_string()
}

/// Trim and collapse runs of whitespace into single spaces. Used for names
/// and header labels where the wiki markup adds newlines/indentation.
pub fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `Str::albionTier($name)`: tier from the item-name prefix.
pub fn albion_tier(name: &str) -> Option<f64> {
    let lower = name.to_lowercase();
    let contains = |needle: &str| lower.contains(needle);

    // Order matters: "Grandmaster's" contains "master's", so it is checked
    // first, exactly like the Laravel macro.
    if contains("beginner's") {
        Some(1.0)
    } else if contains("novice's") {
        Some(2.0)
    } else if contains("journeyman's") {
        Some(3.0)
    } else if contains("adept's") {
        Some(4.0)
    } else if contains("expert's") {
        Some(5.0)
    } else if contains("grandmaster's") {
        Some(7.0)
    } else if contains("master's") {
        Some(6.0)
    } else if contains("elder's") || contains("rosalia's") {
        Some(8.0)
    } else {
        None
    }
}

/// Parses a tier cell such as `"4"`, `"4.0"`, `"Tier 4"` or `"Tier 4.2"`.
pub fn parse_tier_text(text: &str) -> Option<f64> {
    let cleaned = text.trim().trim_start_matches("Tier ").trim();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<f64>().ok().or_else(|| {
        // Some cells carry trailing notes; use the leading number.
        let prefix: String = cleaned
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        prefix.parse::<f64>().ok()
    })
}

/// Enchantment level from the tier text: everything after the first `.`
/// (`"4.0" -> 0`, `"4.3" -> 3`, `"4" -> 0`).
///
/// Note: the Laravel code is `Str::after($tier, '.') ?: 0`, which for a
/// dot-less `"4"` would yield `"4"`; that only occurs for tables that always
/// print `x.0`, and treating it as enchantment 0 is the safe reading.
pub fn enchantment_from_tier(tier_text: &str) -> i64 {
    match tier_text.split_once('.') {
        Some((_, tail)) => tail
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<i64>()
            .unwrap_or(0),
        None => 0,
    }
}

/// `Str::albionIdentifier($icon)`: the file name before the extension, e.g.
/// `https://cdn.../T4_MAIN_AXE.png?x=1` → `T4_MAIN_AXE`.
pub fn identifier_from_icon(icon: &str) -> Option<String> {
    let base = icon.rsplit('/').next()?.split('?').next()?;
    let stem = base.split('.').next()?.trim();
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_string())
    }
}

/// `Str::albionItemPower($text)`: commas removed and parsed as an integer.
pub fn item_power_from_text(text: &str) -> Option<i64> {
    let cleaned: String = text.chars().filter(|c| *c != ',').collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<i64>().ok()
}

/// `Str::wikiPath($path)`: `urldecode` (percent escapes and `+`).
pub fn decode_path(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    let bytes = path.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    Some(String::from_utf8_lossy(&out).into_owned())
}

/// `Str::sanitizeSpellDescription($html)`: strips `class="…"` attributes,
/// leaving the surrounding whitespace exactly as the PHP regex does
/// (`<span class="x">` becomes `<span >`).
pub fn sanitize_spell_description(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let bytes = html.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(rest) = html.get(i..) {
            if let Some(after_prefix) = rest.strip_prefix("class=\"") {
                // Skip through the closing double quote.
                if let Some(end) = after_prefix.find('"') {
                    i += "class=\"".len() + end + 1;
                    continue;
                }
            }
        }
        // Copy one UTF-8 character.
        let ch = html[i..].chars().next().expect("valid boundary");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// `Str::wikiLink($path)`: three known wiki path renames.
pub fn wiki_link(path: &str) -> String {
    match path {
        "/wiki/Hammer" => "/wiki/Hammers".to_string(),
        "/wiki/Bow" => "/wiki/Bows".to_string(),
        "/wiki/Spear" => "/wiki/Spears".to_string(),
        other => other.to_string(),
    }
}

/// `Str::wikiPreview($src)`: absolutise a wiki image path.
pub fn wiki_preview(src: &str) -> String {
    if src.starts_with("http") {
        src.to_string()
    } else {
        format!("https://wiki.albiononline.com{src}")
    }
}

/// Joins a wiki base URL with a path, preserving no double slash.
pub fn wiki_url(base: &str, path: &str) -> String {
    if path.starts_with('/') {
        format!("{}{}", base.trim_end_matches('/'), path)
    } else {
        format!("{}/{}", base.trim_end_matches('/'), path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_from_names_matches_the_laravel_macro() {
        assert_eq!(albion_tier("Beginner's Sword"), Some(1.0));
        assert_eq!(albion_tier("Novice's Mule"), Some(2.0));
        assert_eq!(albion_tier("Journeyman's Battleaxe"), Some(3.0));
        assert_eq!(albion_tier("Adept's Battleaxe"), Some(4.0));
        assert_eq!(albion_tier("Expert's Armored Horse"), Some(5.0));
        assert_eq!(albion_tier("Master's Crossbow"), Some(6.0));
        // "Grandmaster's" must win over "Master's".
        assert_eq!(albion_tier("Grandmaster's Crossbow"), Some(7.0));
        assert_eq!(albion_tier("Elder's Battleaxe"), Some(8.0));
        assert_eq!(albion_tier("Rosalia's Cape"), Some(8.0));
        assert_eq!(albion_tier("Bridgewatch Cape"), None);
    }

    #[test]
    fn tier_text_parsing() {
        assert_eq!(parse_tier_text("4"), Some(4.0));
        assert_eq!(parse_tier_text("4.0"), Some(4.0));
        assert_eq!(parse_tier_text(" 4.2 "), Some(4.2));
        assert_eq!(parse_tier_text("Tier 4"), Some(4.0));
        assert_eq!(parse_tier_text("Tier 7.1"), Some(7.1));
        assert_eq!(parse_tier_text(""), None);
    }

    #[test]
    fn enchantment_comes_after_the_dot() {
        assert_eq!(enchantment_from_tier("4.0"), 0);
        assert_eq!(enchantment_from_tier("4.3"), 3);
        assert_eq!(enchantment_from_tier("4.12"), 12);
        assert_eq!(enchantment_from_tier("4"), 0);
    }

    #[test]
    fn identifier_from_icon_urls() {
        assert_eq!(
            identifier_from_icon(
                "https://render.albiononline.com/v1/item/T4_MAIN_AXE.png?quality=0"
            ),
            Some("T4_MAIN_AXE".to_string())
        );
        assert_eq!(
            identifier_from_icon("//cdn.some/T4_MAIN_AXE@2.png"),
            Some("T4_MAIN_AXE@2".to_string())
        );
        assert_eq!(identifier_from_icon(""), None);
    }

    #[test]
    fn item_power_parsing() {
        assert_eq!(item_power_from_text("700"), Some(700));
        assert_eq!(item_power_from_text("1,100"), Some(1100));
        assert_eq!(item_power_from_text(""), None);
        assert_eq!(item_power_from_text("abc"), None);
    }

    #[test]
    fn path_decoding_handles_percent_and_plus() {
        assert_eq!(
            decode_path("/wiki/Adept%27s_Sword"),
            Some("/wiki/Adept's_Sword".to_string())
        );
        assert_eq!(decode_path("/wiki/A+B"), Some("/wiki/A B".to_string()));
        assert_eq!(decode_path("/wiki/Plain"), Some("/wiki/Plain".to_string()));
        assert_eq!(decode_path(""), None);
    }

    #[test]
    fn spell_description_sanitizing_matches_the_regex() {
        assert_eq!(
            sanitize_spell_description("<span class=\"icon\">x</span>"),
            "<span >x</span>"
        );
        assert_eq!(
            sanitize_spell_description("no classes here"),
            "no classes here"
        );
        // First quote only; rest untouched.
        assert_eq!(
            sanitize_spell_description("<td class=\"a\"><b class=\"b\">x</b></td>"),
            "<td ><b >x</b></td>"
        );
    }

    #[test]
    fn wiki_link_renames() {
        assert_eq!(wiki_link("/wiki/Hammer"), "/wiki/Hammers");
        assert_eq!(wiki_link("/wiki/Bow"), "/wiki/Bows");
        assert_eq!(wiki_link("/wiki/Spear"), "/wiki/Spears");
        assert_eq!(wiki_link("/wiki/Axe"), "/wiki/Axe");
    }

    #[test]
    fn previews_are_absolute() {
        assert_eq!(
            wiki_preview("/images/thumb/x.png"),
            "https://wiki.albiononline.com/images/thumb/x.png"
        );
        assert_eq!(
            wiki_preview("https://example.com/x.png"),
            "https://example.com/x.png"
        );
    }
}
