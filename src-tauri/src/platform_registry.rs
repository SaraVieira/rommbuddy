use std::collections::HashMap;
use std::sync::LazyLock;

/// Definition of a single platform in the registry.
pub struct PlatformDef {
    pub slug: &'static str,
    pub display_name: &'static str,
    pub folder_aliases: &'static [&'static str],
    pub romm_aliases: &'static [&'static str],
    pub dat_aliases: &'static [&'static str],
    pub ra_console_id: Option<u32>,
    pub ss_id: Option<u32>,
    pub libretro_dir: Option<&'static str>,
    pub launchbox_name: Option<&'static str>,
}

/// Central platform registry — single source of truth for all platform data.
pub const PLATFORMS: &[PlatformDef] = &[
    // ── Nintendo ──
    PlatformDef {
        slug: "gb",
        display_name: "Game Boy",
        folder_aliases: &["gb"],
        romm_aliases: &["game-boy"],
        dat_aliases: &["Nintendo - Game Boy"],
        ra_console_id: Some(4),
        ss_id: Some(9),
        libretro_dir: Some("Nintendo - Game Boy"),
        launchbox_name: Some("Nintendo Game Boy"),
    },
    PlatformDef {
        slug: "gbc",
        display_name: "Game Boy Color",
        folder_aliases: &["gbc"],
        romm_aliases: &["game-boy-color"],
        dat_aliases: &["Nintendo - Game Boy Color"],
        ra_console_id: Some(6),
        ss_id: Some(10),
        libretro_dir: Some("Nintendo - Game Boy Color"),
        launchbox_name: Some("Nintendo Game Boy Color"),
    },
    PlatformDef {
        slug: "gba",
        display_name: "Game Boy Advance",
        folder_aliases: &["gba"],
        romm_aliases: &["game-boy-advance"],
        dat_aliases: &["Nintendo - Game Boy Advance"],
        ra_console_id: Some(5),
        ss_id: Some(12),
        libretro_dir: Some("Nintendo - Game Boy Advance"),
        launchbox_name: Some("Nintendo Game Boy Advance"),
    },
    PlatformDef {
        slug: "nes",
        display_name: "NES / Famicom",
        folder_aliases: &["nes", "fc", "famicom"],
        romm_aliases: &["nintendo-entertainment-system", "famicom"],
        dat_aliases: &["Nintendo - Nintendo Entertainment System"],
        ra_console_id: Some(7),
        ss_id: Some(3),
        libretro_dir: Some("Nintendo - Nintendo Entertainment System"),
        launchbox_name: Some("Nintendo Entertainment System"),
    },
    PlatformDef {
        slug: "fds",
        display_name: "Famicom Disk System",
        folder_aliases: &["fds"],
        romm_aliases: &[],
        dat_aliases: &["Nintendo - Famicom Disk System"],
        ra_console_id: None,
        ss_id: Some(106),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "snes",
        display_name: "SNES / Super Famicom",
        folder_aliases: &["snes", "sfc"],
        romm_aliases: &["super-nintendo", "super-famicom", "super-nintendo-entertainment-system", "sfam"],
        dat_aliases: &["Nintendo - Super Nintendo Entertainment System"],
        ra_console_id: Some(3),
        ss_id: Some(4),
        libretro_dir: Some("Nintendo - Super Nintendo Entertainment System"),
        launchbox_name: Some("Super Nintendo Entertainment System"),
    },
    PlatformDef {
        slug: "n64",
        display_name: "Nintendo 64",
        folder_aliases: &["n64"],
        romm_aliases: &["nintendo-64"],
        dat_aliases: &["Nintendo - Nintendo 64"],
        ra_console_id: Some(2),
        ss_id: Some(14),
        libretro_dir: Some("Nintendo - Nintendo 64"),
        launchbox_name: Some("Nintendo 64"),
    },
    PlatformDef {
        slug: "nds",
        display_name: "Nintendo DS",
        folder_aliases: &["nds"],
        romm_aliases: &["nintendo-ds"],
        dat_aliases: &["Nintendo - Nintendo DS"],
        ra_console_id: Some(18),
        ss_id: Some(15),
        libretro_dir: Some("Nintendo - Nintendo DS"),
        launchbox_name: Some("Nintendo DS"),
    },
    PlatformDef {
        slug: "3ds",
        display_name: "Nintendo 3DS",
        folder_aliases: &["3ds"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(17),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "gamecube",
        display_name: "GameCube",
        folder_aliases: &["gamecube", "gc"],
        romm_aliases: &["ngc"],
        dat_aliases: &["Nintendo - GameCube"],
        ra_console_id: None,
        ss_id: Some(13),
        libretro_dir: Some("Nintendo - GameCube"),
        launchbox_name: Some("Nintendo GameCube"),
    },
    PlatformDef {
        slug: "wii",
        display_name: "Wii",
        folder_aliases: &["wii"],
        romm_aliases: &[],
        dat_aliases: &["Nintendo - Wii"],
        ra_console_id: None,
        ss_id: Some(16),
        libretro_dir: Some("Nintendo - Wii"),
        launchbox_name: Some("Nintendo Wii"),
    },
    PlatformDef {
        slug: "wiiu",
        display_name: "Wii U",
        folder_aliases: &["wiiu"],
        romm_aliases: &["wii-u"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(18),
        libretro_dir: Some("Nintendo - Wii U"),
        launchbox_name: Some("Nintendo Wii U"),
    },
    PlatformDef {
        slug: "switch",
        display_name: "Nintendo Switch",
        folder_aliases: &["switch"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(225),
        libretro_dir: None,
        launchbox_name: Some("Nintendo Switch"),
    },
    PlatformDef {
        slug: "switch2",
        display_name: "Nintendo Switch 2",
        folder_aliases: &["switch2"],
        romm_aliases: &["switch-2"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(296),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "dsi",
        display_name: "Nintendo DSi",
        folder_aliases: &["dsi"],
        romm_aliases: &["nintendo-dsi"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(15),
        libretro_dir: None,
        launchbox_name: Some("Nintendo DSi"),
    },
    PlatformDef {
        slug: "n3ds",
        display_name: "New Nintendo 3DS",
        folder_aliases: &["n3ds", "new3ds"],
        romm_aliases: &["new-nintendo-3ds"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(17),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "vb",
        display_name: "Virtual Boy",
        folder_aliases: &["virtualboy", "vb"],
        romm_aliases: &["virtual-boy", "virtualboy"],
        dat_aliases: &["Nintendo - Virtual Boy"],
        ra_console_id: Some(28),
        ss_id: Some(11),
        libretro_dir: Some("Nintendo - Virtual Boy"),
        launchbox_name: Some("Nintendo Virtual Boy"),
    },
    PlatformDef {
        slug: "pokemini",
        display_name: "Pokemon Mini",
        folder_aliases: &["pokemini"],
        romm_aliases: &["pokemon-mini"],
        dat_aliases: &["Nintendo - Pokemon Mini"],
        ra_console_id: None,
        ss_id: Some(211),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "sufami",
        display_name: "Sufami Turbo",
        folder_aliases: &["sufami"],
        romm_aliases: &["sufami-turbo"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(108),
        libretro_dir: None,
        launchbox_name: None,
    },
    // ── Sony ──
    PlatformDef {
        slug: "psx",
        display_name: "PlayStation",
        folder_aliases: &["psx", "ps", "ps1"],
        romm_aliases: &["ps", "playstation", "ps1"],
        dat_aliases: &["Sony - PlayStation"],
        ra_console_id: Some(12),
        ss_id: Some(57),
        libretro_dir: Some("Sony - PlayStation"),
        launchbox_name: Some("Sony Playstation"),
    },
    PlatformDef {
        slug: "ps2",
        display_name: "PlayStation 2",
        folder_aliases: &["ps2"],
        romm_aliases: &["playstation-2"],
        dat_aliases: &["Sony - PlayStation 2"],
        ra_console_id: Some(21),
        ss_id: Some(58),
        libretro_dir: Some("Sony - PlayStation 2"),
        launchbox_name: Some("Sony Playstation 2"),
    },
    PlatformDef {
        slug: "psp",
        display_name: "PlayStation Portable",
        folder_aliases: &["psp"],
        romm_aliases: &["playstation-portable"],
        dat_aliases: &["Sony - PlayStation Portable"],
        ra_console_id: Some(41),
        ss_id: Some(61),
        libretro_dir: Some("Sony - PlayStation Portable"),
        launchbox_name: Some("Sony PSP"),
    },
    PlatformDef {
        slug: "ps3",
        display_name: "PlayStation 3",
        folder_aliases: &["ps3"],
        romm_aliases: &["playstation-3"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(59),
        libretro_dir: Some("Sony - PlayStation 3"),
        launchbox_name: Some("Sony Playstation 3"),
    },
    PlatformDef {
        slug: "ps4",
        display_name: "PlayStation 4",
        folder_aliases: &["ps4"],
        romm_aliases: &["playstation-4"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(60),
        libretro_dir: None,
        launchbox_name: Some("Sony Playstation 4"),
    },
    PlatformDef {
        slug: "ps5",
        display_name: "PlayStation 5",
        folder_aliases: &["ps5"],
        romm_aliases: &["playstation-5"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(284),
        libretro_dir: None,
        launchbox_name: Some("Sony Playstation 5"),
    },
    PlatformDef {
        slug: "psvita",
        display_name: "PlayStation Vita",
        folder_aliases: &["psvita", "vita"],
        romm_aliases: &["playstation-vita"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(62),
        libretro_dir: None,
        launchbox_name: Some("Sony PlayStation Vita"),
    },
    // ── Microsoft ──
    PlatformDef {
        slug: "xbox",
        display_name: "Xbox",
        folder_aliases: &["xbox"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(32),
        libretro_dir: None,
        launchbox_name: Some("Microsoft Xbox"),
    },
    PlatformDef {
        slug: "xbox360",
        display_name: "Xbox 360",
        folder_aliases: &["xbox360"],
        romm_aliases: &["xbox-360"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(33),
        libretro_dir: None,
        launchbox_name: Some("Microsoft Xbox 360"),
    },
    PlatformDef {
        slug: "xboxone",
        display_name: "Xbox One",
        folder_aliases: &["xboxone"],
        romm_aliases: &["xbox-one"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(34),
        libretro_dir: None,
        launchbox_name: Some("Microsoft Xbox One"),
    },
    PlatformDef {
        slug: "xboxseriesx",
        display_name: "Xbox Series X/S",
        folder_aliases: &["xboxseriesx"],
        romm_aliases: &["series-x-s"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: None,
        libretro_dir: None,
        launchbox_name: None,
    },
    // ── Sega ──
    PlatformDef {
        slug: "genesis",
        display_name: "Sega Genesis / Mega Drive",
        folder_aliases: &["genesis", "megadrive", "md"],
        romm_aliases: &["megadrive", "mega-drive", "sega-genesis", "mega-drive-slash-genesis"],
        dat_aliases: &["Sega - Mega Drive - Genesis"],
        ra_console_id: Some(1),
        ss_id: Some(1),
        libretro_dir: Some("Sega - Mega Drive - Genesis"),
        launchbox_name: Some("Sega Genesis"),
    },
    PlatformDef {
        slug: "segacd",
        display_name: "Sega CD",
        folder_aliases: &["segacd"],
        romm_aliases: &["sega-cd"],
        dat_aliases: &["Sega - Mega-CD - Sega CD"],
        ra_console_id: Some(9),
        ss_id: Some(20),
        libretro_dir: Some("Sega - Mega-CD - Sega CD"),
        launchbox_name: Some("Sega CD"),
    },
    PlatformDef {
        slug: "saturn",
        display_name: "Sega Saturn",
        folder_aliases: &["saturn"],
        romm_aliases: &["sega-saturn"],
        dat_aliases: &["Sega - Saturn"],
        ra_console_id: Some(39),
        ss_id: Some(22),
        libretro_dir: Some("Sega - Saturn"),
        launchbox_name: Some("Sega Saturn"),
    },
    PlatformDef {
        slug: "dreamcast",
        display_name: "Dreamcast",
        folder_aliases: &["dreamcast", "dc"],
        romm_aliases: &["sega-dreamcast", "dc"],
        dat_aliases: &["Sega - Dreamcast"],
        ra_console_id: Some(40),
        ss_id: Some(23),
        libretro_dir: Some("Sega - Dreamcast"),
        launchbox_name: Some("Sega Dreamcast"),
    },
    PlatformDef {
        slug: "gamegear",
        display_name: "Game Gear",
        folder_aliases: &["gamegear", "gg"],
        romm_aliases: &["game-gear"],
        dat_aliases: &["Sega - Game Gear"],
        ra_console_id: Some(15),
        ss_id: Some(21),
        libretro_dir: Some("Sega - Game Gear"),
        launchbox_name: Some("Sega Game Gear"),
    },
    PlatformDef {
        slug: "mastersystem",
        display_name: "Master System",
        folder_aliases: &["mastersystem", "ms", "sms"],
        romm_aliases: &["master-system", "sega-master-system", "sms"],
        dat_aliases: &["Sega - Master System - Mark III"],
        ra_console_id: Some(11),
        ss_id: Some(2),
        libretro_dir: Some("Sega - Master System - Mark III"),
        launchbox_name: Some("Sega Master System"),
    },
    PlatformDef {
        slug: "sg1000",
        display_name: "SG-1000",
        folder_aliases: &["sg-1000", "sg1000", "sg"],
        romm_aliases: &[],
        dat_aliases: &["Sega - SG-1000"],
        ra_console_id: Some(33),
        ss_id: Some(109),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "sega32",
        display_name: "Sega 32X",
        folder_aliases: &["sega32", "32x"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: Some(10),
        ss_id: Some(19),
        libretro_dir: None,
        launchbox_name: None,
    },
    // ── Capcom Arcade ──
    PlatformDef {
        slug: "cps1",
        display_name: "Capcom Play System",
        folder_aliases: &["cps1"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(6),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "cps2",
        display_name: "Capcom Play System 2",
        folder_aliases: &["cps2"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(7),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "cps3",
        display_name: "Capcom Play System 3",
        folder_aliases: &["cps3"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(8),
        libretro_dir: None,
        launchbox_name: None,
    },
    // ── SNK / Arcade ──
    PlatformDef {
        slug: "neogeo",
        display_name: "Neo Geo",
        folder_aliases: &["neogeo"],
        romm_aliases: &["neo-geo-aes", "neogeoaes", "neo-geo-mvs", "neogeomvs"],
        dat_aliases: &["SNK - Neo Geo"],
        ra_console_id: Some(14),
        ss_id: None,
        libretro_dir: Some("SNK - Neo Geo"),
        launchbox_name: Some("SNK Neo Geo AES"),
    },
    PlatformDef {
        slug: "arcade",
        display_name: "Arcade",
        folder_aliases: &["arcade", "mame", "fbneo", "fba"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: Some(27),
        ss_id: Some(75),
        libretro_dir: Some("MAME"),
        launchbox_name: Some("Arcade"),
    },
    PlatformDef {
        slug: "ngp",
        display_name: "Neo Geo Pocket",
        folder_aliases: &["ngp"],
        romm_aliases: &["neo-geo-pocket"],
        dat_aliases: &["SNK - Neo Geo Pocket"],
        ra_console_id: Some(14),
        ss_id: Some(25),
        libretro_dir: Some("SNK - Neo Geo Pocket"),
        launchbox_name: Some("SNK Neo Geo Pocket"),
    },
    PlatformDef {
        slug: "ngpc",
        display_name: "Neo Geo Pocket Color",
        folder_aliases: &["ngpc"],
        romm_aliases: &["neo-geo-pocket-color"],
        dat_aliases: &["SNK - Neo Geo Pocket Color"],
        ra_console_id: Some(14),
        ss_id: Some(82),
        libretro_dir: Some("SNK - Neo Geo Pocket Color"),
        launchbox_name: Some("SNK Neo Geo Pocket Color"),
    },
    PlatformDef {
        slug: "neocd",
        display_name: "Neo Geo CD",
        folder_aliases: &["neocd"],
        romm_aliases: &["neo-geo-cd"],
        dat_aliases: &["SNK - Neo Geo CD"],
        ra_console_id: None,
        ss_id: Some(70),
        libretro_dir: None,
        launchbox_name: None,
    },
    // ── NEC ──
    PlatformDef {
        slug: "pce",
        display_name: "TurboGrafx-16 / PC Engine",
        folder_aliases: &["pcengine", "pce", "tg16"],
        romm_aliases: &["turbografx-16", "tg16", "pc-engine"],
        dat_aliases: &["NEC - PC Engine - TurboGrafx-16"],
        ra_console_id: Some(8),
        ss_id: Some(31),
        libretro_dir: Some("NEC - PC Engine - TurboGrafx 16"),
        launchbox_name: Some("NEC TurboGrafx-16"),
    },
    PlatformDef {
        slug: "pcecd",
        display_name: "TurboGrafx-CD",
        folder_aliases: &["pcenginecd", "pcecd", "tgcd"],
        romm_aliases: &["turbografx-cd", "tg-cd", "pc-engine-cd"],
        dat_aliases: &["NEC - PC Engine CD - TurboGrafx-CD"],
        ra_console_id: Some(76),
        ss_id: Some(114),
        libretro_dir: Some("NEC - PC Engine CD - TurboGrafx-CD"),
        launchbox_name: Some("NEC TurboGrafx-CD"),
    },
    PlatformDef {
        slug: "sgfx",
        display_name: "SuperGrafx",
        folder_aliases: &["supergrafx", "sgfx"],
        romm_aliases: &["supergrafx"],
        dat_aliases: &["NEC - PC Engine SuperGrafx"],
        ra_console_id: None,
        ss_id: Some(105),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "pcfx",
        display_name: "PC-FX",
        folder_aliases: &["pcfx"],
        romm_aliases: &["pc-fx"],
        dat_aliases: &["NEC - PC-FX"],
        ra_console_id: None,
        ss_id: Some(72),
        libretro_dir: None,
        launchbox_name: None,
    },
    // ── Atari ──
    PlatformDef {
        slug: "atari2600",
        display_name: "Atari 2600",
        folder_aliases: &["atari2600", "atari", "a26"],
        romm_aliases: &[],
        dat_aliases: &["Atari - 2600"],
        ra_console_id: Some(25),
        ss_id: Some(26),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "atari5200",
        display_name: "Atari 5200",
        folder_aliases: &["atari5200"],
        romm_aliases: &[],
        dat_aliases: &["Atari - 5200"],
        ra_console_id: None,
        ss_id: Some(40),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "atari7800",
        display_name: "Atari 7800",
        folder_aliases: &["atari7800", "a78"],
        romm_aliases: &[],
        dat_aliases: &["Atari - 7800"],
        ra_console_id: Some(51),
        ss_id: Some(41),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "lynx",
        display_name: "Atari Lynx",
        folder_aliases: &["lynx"],
        romm_aliases: &["atari-lynx"],
        dat_aliases: &["Atari - Lynx"],
        ra_console_id: Some(13),
        ss_id: Some(28),
        libretro_dir: Some("Atari - Lynx"),
        launchbox_name: Some("Atari Lynx"),
    },
    PlatformDef {
        slug: "atarist",
        display_name: "Atari ST",
        folder_aliases: &["atarist"],
        romm_aliases: &["atari-st"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(42),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "jaguar",
        display_name: "Atari Jaguar",
        folder_aliases: &["jaguar"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: Some(17),
        ss_id: Some(27),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "atari8bit",
        display_name: "Atari 8-bit",
        folder_aliases: &["atari8bit", "atari800"],
        romm_aliases: &["atari800"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(43),
        libretro_dir: None,
        launchbox_name: None,
    },
    // ── Bandai ──
    PlatformDef {
        slug: "ws",
        display_name: "WonderSwan",
        folder_aliases: &["wonderswan", "ws"],
        romm_aliases: &["wonderswan"],
        dat_aliases: &["Bandai - WonderSwan"],
        ra_console_id: Some(53),
        ss_id: Some(45),
        libretro_dir: Some("Bandai - WonderSwan"),
        launchbox_name: Some("WonderSwan"),
    },
    PlatformDef {
        slug: "wsc",
        display_name: "WonderSwan Color",
        folder_aliases: &["wonderswancolor", "wsc"],
        romm_aliases: &["wonderswan-color"],
        dat_aliases: &["Bandai - WonderSwan Color"],
        ra_console_id: Some(53),
        ss_id: Some(46),
        libretro_dir: Some("Bandai - WonderSwan Color"),
        launchbox_name: Some("WonderSwan Color"),
    },
    // ── Other consoles ──
    PlatformDef {
        slug: "colecovision",
        display_name: "ColecoVision",
        folder_aliases: &["coleco", "colecovision", "col"],
        romm_aliases: &[],
        dat_aliases: &["Coleco - ColecoVision"],
        ra_console_id: Some(44),
        ss_id: Some(48),
        libretro_dir: Some("Coleco - ColecoVision"),
        launchbox_name: Some("ColecoVision"),
    },
    PlatformDef {
        slug: "intellivision",
        display_name: "Intellivision",
        folder_aliases: &["intellivision", "int"],
        romm_aliases: &[],
        dat_aliases: &["Mattel - Intellivision"],
        ra_console_id: Some(45),
        ss_id: Some(115),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "vectrex",
        display_name: "Vectrex",
        folder_aliases: &["vectrex"],
        romm_aliases: &[],
        dat_aliases: &["GCE - Vectrex"],
        ra_console_id: None,
        ss_id: Some(102),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "channelf",
        display_name: "Channel F",
        folder_aliases: &["channelf"],
        romm_aliases: &["fairchild-channel-f"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(80),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "3do",
        display_name: "3DO Interactive Multiplayer",
        folder_aliases: &["3do"],
        romm_aliases: &[],
        dat_aliases: &["Panasonic - 3DO Interactive Multiplayer"],
        ra_console_id: Some(43),
        ss_id: Some(29),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "cdi",
        display_name: "Philips CD-i",
        folder_aliases: &["cdi"],
        romm_aliases: &["philips-cd-i"],
        dat_aliases: &["Philips - CD-i"],
        ra_console_id: None,
        ss_id: Some(133),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "odyssey2",
        display_name: "Odyssey 2 / Videopac",
        folder_aliases: &["odyssey2"],
        romm_aliases: &["odyssey-2"],
        dat_aliases: &[],
        ra_console_id: Some(23),
        ss_id: Some(104),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "megaduck",
        display_name: "Mega Duck",
        folder_aliases: &["megaduck"],
        romm_aliases: &["mega-duck-slash-cougar-boy"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: None,
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "supervision",
        display_name: "Watara Supervision",
        folder_aliases: &["supervision"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(207),
        libretro_dir: None,
        launchbox_name: None,
    },
    // ── Computers ──
    PlatformDef {
        slug: "win",
        display_name: "PC (Windows)",
        folder_aliases: &["win", "windows"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(138),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "msx",
        display_name: "MSX",
        folder_aliases: &["msx"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(113),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "msx2",
        display_name: "MSX2",
        folder_aliases: &["msx2"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(116),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "dos",
        display_name: "DOS",
        folder_aliases: &["dos"],
        romm_aliases: &["ms-dos", "msdos"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(135),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "cpc",
        display_name: "Amstrad CPC",
        folder_aliases: &["amstradcpc", "cpc"],
        romm_aliases: &["acpc", "amstrad-cpc"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(65),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "zxspectrum",
        display_name: "ZX Spectrum",
        folder_aliases: &["zxspectrum"],
        romm_aliases: &["zx-spectrum", "zxspectrum", "zxs"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(76),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "c64",
        display_name: "Commodore 64",
        folder_aliases: &["c64"],
        romm_aliases: &["commodore-64"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(66),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "amiga",
        display_name: "Amiga",
        folder_aliases: &["amiga"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(64),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "scummvm",
        display_name: "ScummVM",
        folder_aliases: &["scummvm"],
        romm_aliases: &[],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(123),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "vic20",
        display_name: "VIC-20",
        folder_aliases: &["vic20"],
        romm_aliases: &["vic-20"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(73),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "x68000",
        display_name: "Sharp X68000",
        folder_aliases: &["x68000"],
        romm_aliases: &["sharp-x68000"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(79),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "pc98",
        display_name: "PC-9800 Series",
        folder_aliases: &["pc98"],
        romm_aliases: &["pc-9800-series"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(208),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "trs80",
        display_name: "TRS-80",
        folder_aliases: &["trs80"],
        romm_aliases: &["trs-80"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: None,
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "ti99",
        display_name: "TI-99",
        folder_aliases: &["ti99"],
        romm_aliases: &["ti-99"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(205),
        libretro_dir: None,
        launchbox_name: None,
    },
    // ── Fantasy Consoles ──
    PlatformDef {
        slug: "tic80",
        display_name: "TIC-80",
        folder_aliases: &["tic80", "tic-80"],
        romm_aliases: &["tic-80"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(222),
        libretro_dir: None,
        launchbox_name: None,
    },
    PlatformDef {
        slug: "pico8",
        display_name: "PICO-8",
        folder_aliases: &["pico8", "pico-8"],
        romm_aliases: &["pico"],
        dat_aliases: &[],
        ra_console_id: None,
        ss_id: Some(234),
        libretro_dir: None,
        launchbox_name: None,
    },
];

// ── Derived lookup maps ──

/// Folder name (lowercase) → canonical slug.
static FOLDER_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for p in PLATFORMS {
        // The slug itself is always a valid folder name
        m.insert(p.slug, p.slug);
        for &alias in p.folder_aliases {
            m.insert(alias, p.slug);
        }
    }
    m
});

/// ROMM slug (lowercase) → canonical slug.
static ROMM_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for p in PLATFORMS {
        for &alias in p.romm_aliases {
            m.insert(alias, p.slug);
        }
    }
    m
});

/// DAT header name → canonical slug.
static DAT_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for p in PLATFORMS {
        for &alias in p.dat_aliases {
            m.insert(alias, p.slug);
        }
    }
    m
});

/// Canonical slug → display name.
static DISPLAY_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    PLATFORMS.iter().map(|p| (p.slug, p.display_name)).collect()
});

/// Canonical slug → RetroAchievements console ID.
static RA_MAP: LazyLock<HashMap<&'static str, u32>> = LazyLock::new(|| {
    PLATFORMS
        .iter()
        .filter_map(|p| p.ra_console_id.map(|id| (p.slug, id)))
        .collect()
});

/// Canonical slug → ScreenScraper system ID.
static SS_MAP: LazyLock<HashMap<&'static str, u32>> = LazyLock::new(|| {
    PLATFORMS
        .iter()
        .filter_map(|p| p.ss_id.map(|id| (p.slug, id)))
        .collect()
});

/// Canonical slug → libretro thumbnail directory name.
static LIBRETRO_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    PLATFORMS
        .iter()
        .filter_map(|p| p.libretro_dir.map(|d| (p.slug, d)))
        .collect()
});

/// Canonical slug → LaunchBox platform name.
static LAUNCHBOX_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    PLATFORMS
        .iter()
        .filter_map(|p| p.launchbox_name.map(|n| (p.slug, n)))
        .collect()
});

// ── Public convenience functions ──

/// Resolve a folder name to a canonical platform slug.
pub fn resolve_folder(name: &str) -> Option<&'static str> {
    FOLDER_MAP.get(name).copied()
}

/// Check if a folder name is a known platform.
pub fn is_known_folder(name: &str) -> bool {
    FOLDER_MAP.contains_key(name)
}

/// Resolve a ROMM platform slug to our canonical slug.
/// Returns the mapped slug if one exists, otherwise returns the ROMM slug as-is.
pub fn resolve_romm_slug(romm_slug: &str) -> String {
    let lower = romm_slug.to_lowercase();
    if let Some(&canonical) = ROMM_MAP.get(lower.as_str()) {
        canonical.to_string()
    } else {
        lower
    }
}

/// Resolve a DAT header name to a canonical platform slug.
pub fn resolve_dat_name(dat_name: &str) -> Option<&'static str> {
    DAT_MAP.get(dat_name).copied()
}

/// Get the display name for a canonical platform slug.
pub fn display_name(slug: &str) -> Option<&'static str> {
    DISPLAY_MAP.get(slug).copied()
}

/// Get the RetroAchievements console ID for a canonical platform slug.
pub fn ra_console_id(slug: &str) -> Option<u32> {
    RA_MAP.get(slug).copied()
}

/// Get the ScreenScraper system ID for a canonical platform slug.
pub fn ss_id(slug: &str) -> Option<u32> {
    SS_MAP.get(slug).copied()
}

/// Get the libretro thumbnail directory name for a canonical platform slug.
pub fn libretro_dir(slug: &str) -> Option<&'static str> {
    LIBRETRO_MAP.get(slug).copied()
}

/// Get the LaunchBox platform name for a canonical platform slug.
pub fn launchbox_name(slug: &str) -> Option<&'static str> {
    LAUNCHBOX_MAP.get(slug).copied()
}
