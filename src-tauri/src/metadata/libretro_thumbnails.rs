use std::fmt::Write;

use crate::platform_registry;

/// Sanitize a game name for use in a libretro thumbnail URL.
/// Matches `RetroArch`'s character replacement: `&*/:`\"`<>?\|` -> `_`
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '&' | '*' | '/' | ':' | '`' | '"' | '<' | '>' | '?' | '\\' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Percent-encode a string for URL path segments.
fn encode_uri_component(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                let _ = write!(result, "%{byte:02X}");
            }
        }
    }
    result
}

/// Build a libretro thumbnail URL for the given platform, game name, and category.
/// Returns `None` if the platform is not in the slug map.
fn build_url(platform_slug: &str, game_name: &str, category: &str) -> Option<String> {
    let system = platform_registry::libretro_dir(platform_slug)?;
    let sanitized = sanitize_name(game_name);
    Some(format!(
        "https://thumbnails.libretro.com/{}/{}/{}.png",
        encode_uri_component(system),
        category,
        encode_uri_component(&sanitized)
    ))
}

/// Build a libretro boxart/cover thumbnail URL.
pub fn build_thumbnail_url(platform_slug: &str, game_name: &str) -> Option<String> {
    build_url(platform_slug, game_name, "Named_Boxarts")
}

/// Build a libretro in-game snapshot URL.
pub fn build_snapshot_url(platform_slug: &str, game_name: &str) -> Option<String> {
    build_url(platform_slug, game_name, "Named_Snaps")
}

/// Build a libretro title screen URL.
pub fn build_title_url(platform_slug: &str, game_name: &str) -> Option<String> {
    build_url(platform_slug, game_name, "Named_Titles")
}
