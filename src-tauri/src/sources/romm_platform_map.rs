use std::collections::HashMap;
use std::sync::LazyLock;

/// Maps ROMM platform slugs to our canonical DB slugs where they differ.
/// Only needed when ROMM uses a different slug than what we have in the DB.
/// Platforms not in this map will be auto-created using their `ROMM` slug.
static ROMM_TO_CANONICAL: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // ── Nintendo ──
    m.insert("game-boy", "gb");
    m.insert("game-boy-color", "gbc");
    m.insert("game-boy-advance", "gba");
    m.insert("nintendo-entertainment-system", "nes");
    m.insert("famicom", "nes");
    m.insert("super-nintendo", "snes");
    m.insert("super-famicom", "snes");
    m.insert("super-nintendo-entertainment-system", "snes");
    m.insert("nintendo-64", "n64");
    m.insert("nintendo-ds", "nds");

    // ── Sony ──
    m.insert("ps", "psx");
    m.insert("playstation", "psx");
    m.insert("ps1", "psx");
    m.insert("playstation-2", "ps2");
    m.insert("playstation-portable", "psp");

    // ── Sega ──
    m.insert("megadrive", "genesis");
    m.insert("mega-drive", "genesis");
    m.insert("sega-genesis", "genesis");
    m.insert("mega-drive-slash-genesis", "genesis");
    m.insert("sega-cd", "segacd");
    m.insert("sega-saturn", "saturn");
    m.insert("sega-dreamcast", "dreamcast");
    m.insert("dc", "dreamcast");
    m.insert("game-gear", "gamegear");
    m.insert("master-system", "mastersystem");
    m.insert("sega-master-system", "mastersystem");
    m.insert("sms", "mastersystem");

    // ── SNK / Arcade ──
    m.insert("neo-geo-aes", "neogeo");
    m.insert("neogeoaes", "neogeo");
    m.insert("neo-geo-mvs", "neogeo");
    m.insert("neogeomvs", "neogeo");
    m.insert("neo-geo-pocket", "ngp");
    m.insert("neo-geo-pocket-color", "ngpc");

    // ── NEC ──
    m.insert("turbografx-16", "pce");
    m.insert("tg16", "pce");
    m.insert("pc-engine", "pce");
    m.insert("turbografx-cd", "pcecd");
    m.insert("tg-cd", "pcecd");
    m.insert("pc-engine-cd", "pcecd");
    m.insert("pc-fx", "pcfx");

    // ── Atari ──
    m.insert("atari-lynx", "lynx");
    m.insert("atari-st", "atarist");

    // ── Nintendo (misc) ──
    m.insert("virtual-boy", "vb");
    m.insert("wonderswan", "ws");
    m.insert("wonderswan-color", "wsc");

    // ── Amstrad ──
    m.insert("acpc", "cpc");
    m.insert("amstrad-cpc", "cpc");

    // ── Commodore ──
    m.insert("commodore-64", "c64");
    m.insert("vic-20", "vic20");

    // ── Other ──
    m.insert("ms-dos", "dos");
    m.insert("msdos", "dos");
    m.insert("zx-spectrum", "zxspectrum");
    m.insert("zxspectrum", "zxspectrum");
    m.insert("sharp-x68000", "x68000");
    m.insert("pc-9800-series", "pc98");
    m.insert("trs-80", "trs80");
    m.insert("ti-99", "ti99");

    m
});

/// Resolve a `ROMM` platform slug to our canonical slug.
/// Returns the mapped slug if one exists, otherwise returns the `ROMM` slug as-is
/// (the platform will be auto-created in the DB during sync).
pub fn resolve_platform_slug(romm_slug: &str) -> String {
    let lower = romm_slug.to_lowercase();

    if let Some(&canonical) = ROMM_TO_CANONICAL.get(lower.as_str()) {
        canonical.to_string()
    } else {
        lower
    }
}
