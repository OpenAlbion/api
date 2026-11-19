//! Small helpers that replicate PHP behaviors used by the Laravel app.

/// Replicates the `tier` accessor:
/// `fn ($value) => $value ? number_format($value, 1) : null`
///
/// `number_format` returns a *string* with one decimal, and PHP treats a
/// falsy value (null / 0) as null.
pub fn tier_string(tier: Option<f64>) -> Option<String> {
    match tier {
        Some(t) if t != 0.0 => Some(format!("{:.1}", t)),
        _ => None,
    }
}

/// Slot display names, mirroring `App\Enums\SpellSlot::slotName()`.
pub fn slot_name(slot: &str) -> String {
    match slot {
        "q" => "First Slot",
        "w" => "Second Slot",
        "e" => "Third Slot",
        "d" => "Fourth Slot",
        "r" => "Fifth Slot",
        "f" => "Sixth Slot",
        "mount" => "Mount",
        "passive" => "Passive",
        other => other,
    }
    .to_string()
}

/// Pluralized category type used for the `path` field on weaponry routes,
/// mirroring `Str::plural($this->type->value)`.
pub fn plural_type(item_type: &str) -> String {
    match item_type {
        "weapon" => "weapons",
        "armor" => "armors",
        "accessory" => "accessories",
        "mount" => "mounts",
        "consumable" => "consumables",
        other => other,
    }
    .to_string()
}

/// PHP `strip_tags` equivalent for the spell `description` field: removes
/// `<...>` spans, leaving the text content (entities are left encoded,
/// exactly like PHP). An unterminated tag consumes the rest of the string.
pub fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        if !in_tag && ch == '<' {
            in_tag = true;
        } else if in_tag {
            if ch == '>' {
                in_tag = false;
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Laravel's `when($request->input(...))` semantics: a present-but-falsy
/// query value (`""` or `"0"`) means "no filter", same as a missing value.
/// Returns `Ok(Some(value))` when the filter applies, `Ok(None)` when it
/// should be skipped, and `Err(())` when the value is present and truthy but
/// not a number (SQLite numeric-affinity comparison then matches nothing).
#[allow(clippy::result_unit_err)]
pub fn php_query_number(raw: &Option<String>) -> Result<Option<i64>, ()> {
    match raw {
        None => Ok(None),
        Some(v) if v.is_empty() || v == "0" => Ok(None),
        Some(v) => match v.parse::<i64>() {
            Ok(n) => Ok(Some(n)),
            Err(_) => match v.parse::<f64>() {
                // A float like "4.0" — keep the float path out of the int
                // filter by matching on the parsed value below.
                Ok(_) => Err(()),
                Err(_) => Err(()),
            },
        },
    }
}

/// Like [`php_query_number`] but for float columns (`tier`).
#[allow(clippy::result_unit_err)]
pub fn php_query_float(raw: &Option<String>) -> Result<Option<f64>, ()> {
    match raw {
        None => Ok(None),
        Some(v) if v.is_empty() || v == "0" => Ok(None),
        Some(v) => v
            .parse::<f64>()
            .map(Some)
            .map_err(|_| ()),
    }
}

/// Truthy check for plain string filters (e.g. categories `type`):
/// Laravel skips the filter for missing / "" / "0".
pub fn php_query_string(raw: &Option<String>) -> Option<String> {
    match raw {
        None => None,
        Some(v) if v.is_empty() || v == "0" => None,
        Some(v) => Some(v.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_is_formatted_like_php() {
        assert_eq!(tier_string(Some(4.0)).as_deref(), Some("4.0"));
        assert_eq!(tier_string(Some(3.0)).as_deref(), Some("3.0"));
        assert_eq!(tier_string(Some(4.5)).as_deref(), Some("4.5"));
        assert_eq!(tier_string(Some(0.0)), None);
        assert_eq!(tier_string(None), None);
    }

    #[test]
    fn slot_names_match_the_enum() {
        assert_eq!(slot_name("q"), "First Slot");
        assert_eq!(slot_name("w"), "Second Slot");
        assert_eq!(slot_name("e"), "Third Slot");
        assert_eq!(slot_name("d"), "Fourth Slot");
        assert_eq!(slot_name("r"), "Fifth Slot");
        assert_eq!(slot_name("f"), "Sixth Slot");
        assert_eq!(slot_name("mount"), "Mount");
        assert_eq!(slot_name("passive"), "Passive");
    }

    #[test]
    fn plurals_match_str_plural() {
        assert_eq!(plural_type("weapon"), "weapons");
        assert_eq!(plural_type("armor"), "armors");
        assert_eq!(plural_type("accessory"), "accessories");
        assert_eq!(plural_type("mount"), "mounts");
        assert_eq!(plural_type("consumable"), "consumables");
    }

    #[test]
    fn strips_tags_like_php() {
        assert_eq!(strip_tags("<b>bold</b> text"), "bold text");
        assert_eq!(strip_tags("a <span >tag</span>"), "a tag");
        assert_eq!(strip_tags("no tags"), "no tags");
        assert_eq!(strip_tags("unterminated <tag"), "unterminated ");
        assert_eq!(strip_tags("entities &amp; stay"), "entities &amp; stay");
    }

    #[test]
    fn query_number_follows_php_falsy_rules() {
        assert_eq!(php_query_number(&None), Ok(None));
        assert_eq!(php_query_number(&Some(String::new())), Ok(None));
        assert_eq!(php_query_number(&Some("0".into())), Ok(None));
        assert_eq!(php_query_number(&Some("3".into())), Ok(Some(3)));
        assert!(php_query_number(&Some("abc".into())).is_err());
    }

    #[test]
    fn query_float_parses_decimals() {
        assert_eq!(php_query_float(&Some("4".into())), Ok(Some(4.0)));
        assert_eq!(php_query_float(&Some("4.0".into())), Ok(Some(4.0)));
        assert_eq!(php_query_float(&Some("0".into())), Ok(None));
        assert!(php_query_float(&Some("x".into())).is_err());
    }
}
