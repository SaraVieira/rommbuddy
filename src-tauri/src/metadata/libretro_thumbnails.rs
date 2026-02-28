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

/// Build a libretro thumbnail URL for the given platform slug and game name.
/// Returns `None` if the platform is not in the slug map.
pub fn build_thumbnail_url(platform_slug: &str, game_name: &str) -> Option<String> {
    let system = platform_registry::libretro_dir(platform_slug)?;
    let sanitized = sanitize_name(game_name);
    Some(format!(
        "https://thumbnails.libretro.com/{}/Named_Boxarts/{}.png",
        encode_uri_component(system),
        encode_uri_component(&sanitized)
    ))
}

/// Build a libretro in-game snapshot URL for the given platform slug and game name.
/// Returns `None` if the platform is not in the slug map.
pub fn build_snapshot_url(platform_slug: &str, game_name: &str) -> Option<String> {
    let system = platform_registry::libretro_dir(platform_slug)?;
    let sanitized = sanitize_name(game_name);
    Some(format!(
        "https://thumbnails.libretro.com/{}/Named_Snaps/{}.png",
        encode_uri_component(system),
        encode_uri_component(&sanitized)
    ))
}

/// Build a libretro title screen URL for the given platform slug and game name.
/// Returns `None` if the platform is not in the slug map.
pub fn build_title_url(platform_slug: &str, game_name: &str) -> Option<String> {
    let system = platform_registry::libretro_dir(platform_slug)?;
    let sanitized = sanitize_name(game_name);
    Some(format!(
        "https://thumbnails.libretro.com/{}/Named_Titles/{}.png",
        encode_uri_component(system),
        encode_uri_component(&sanitized)
    ))
}
