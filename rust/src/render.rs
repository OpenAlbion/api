//! Ports `App\Services\Render\RenderService`.
//!
//! The Laravel service has (buggy) setters that actually overwrite
//! `quality`; only the enchantment flow is used by the API, so the builders
//! here expose exactly the combinations the resources use.

pub const SIZE: u32 = 217;
pub const LOCALE: &str = "en";

/// `renderItem($identifier)` with the default enchantment 0 / quality 0.
pub fn render_item(identifier: &str) -> String {
    render_item_enchanted(identifier, 0)
}

/// `renderItem($identifier)` after `setEnchantment($n)`.
pub fn render_item_enchanted(identifier: &str, enchantment: i64) -> String {
    format!(
        "https://render.albiononline.com/v1/item/{identifier}@{enchantment}.png?quality=0&size={size}&locale={locale}",
        identifier = identifier,
        enchantment = enchantment,
        size = SIZE,
        locale = LOCALE,
    )
}

/// `renderSpell($identifier)`.
pub fn render_spell(identifier: &str) -> String {
    format!(
        "https://render.albiononline.com/v1/spell/{identifier}.png?size={size}&locale={locale}",
        identifier = identifier,
        size = SIZE,
        locale = LOCALE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_urls_match_php_output() {
        assert_eq!(
            render_item("T4_MAIN_AXE"),
            "https://render.albiononline.com/v1/item/T4_MAIN_AXE@0.png?quality=0&size=217&locale=en"
        );
        assert_eq!(
            render_item_enchanted("T4_MAIN_AXE", 2),
            "https://render.albiononline.com/v1/item/T4_MAIN_AXE@2.png?quality=0&size=217&locale=en"
        );
    }

    #[test]
    fn spell_urls_match_php_output() {
        assert_eq!(
            render_spell("Rending Strike"),
            "https://render.albiononline.com/v1/spell/Rending Strike.png?size=217&locale=en"
        );
    }
}
