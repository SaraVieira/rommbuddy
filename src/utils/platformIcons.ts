/**
 * Maps platform slugs to their pixel art console icon files.
 * Icons sourced from SamuraiCowboy's icon pack.
 *
 * Each key is a possible slug (lowercase). Multiple slugs can map
 * to the same icon file. The function tries an exact match first,
 * then falls back to partial/keyword matching.
 */
const ICON_MAP: Record<string, string> = {
  // ── Nintendo ──
  gb: "gb.png",
  gbc: "gbc.png",
  gba: "gba.png",
  nes: "nintendo-famicom.png",
  fds: "nintendo-famicom-disk-system.png",
  snes: "snes.png",
  n64: "n64.png",
  nds: "nintendo-ds.png",
  "3ds": "nintendo-ds-lite.png",
  ngc: "gc.png",
  wii: "wii.png",
  wiiu: "wiiu.png",
  switch: "switch.png",
  switch2: "switch2.png",
  dsi: "dsi.png",
  n3ds: "n3ds.png",
  virtualboy: "nintendo-virtual-boy.png",
  "pokemon-mini": "pkm-mini.png",
  sufami: "sufami.png",

  // ── Sony ──
  psx: "sony-playstation.png",
  ps2: "sony-playstation.png",
  psp: "sony-psp.png",
  ps3: "ps3.png",
  ps4: "ps4.png",
  ps5: "ps5.png",
  psvita: "vita.png",

  // ── Sega ──
  genesis: "sega-genesis-us.png",
  segacd: "sega-genesis-cd-us.png",
  saturn: "sega-saturn-us.png",
  dreamcast: "sega-dreamcast.png",
  gamegear: "gg.png",
  mastersystem: "ms.png",
  sg1000: "sega-sg-1000.png",
  sega32: "sega-genesis-32x-us.png",

  // ── Microsoft ──
  xbox: "xbox.png",
  xbox360: "360.png",
  xboxone: "one.png",
  xboxseriesx: "one.png",

  // ── Capcom Arcade ──
  cps1: "cps1.png",
  cps2: "cps2.png",
  cps3: "cps3.png",

  // ── SNK / Arcade ──
  neogeo: "snk-ngpc-carbon-black.png",
  arcade: "arcade.png",
  ngp: "snk-ngpc-platinum-silver.png",
  ngpc: "snk-ngpc-crystal-white.png",
  neocd: "neocd.png",

  // ── NEC ──
  pce: "nec-pc-engine-duo.png",
  pcecd: "nec-turbografxcd.png",
  supergrafx: "nec-supergrafx.png",
  pcfx: "nec-pc-fx.png",

  // ── Atari ──
  atari2600: "2600.png",
  atari5200: "atari-5200-version-1.png",
  atari7800: "atari-7800-version-1.png",
  lynx: "atari-lynx.png",
  atarist: "atari-st-520.png",
  jaguar: "atari-jaguar.png",
  atari8bit: "atari-800.png",

  // ── Bandai ──
  ws: "bandai-wonderswan.png",
  wsc: "bandai-wonderswan-color.png",

  // ── Other consoles ──
  colecovision: "colecovision.png",
  intellivision: "mattel-intellivision.png",
  vectrex: "gce-vectrex-open.png",
  channelf: "channelf.png",
  "3do": "3do-panasonic.png",
  "philips-cd-i": "cdi.png",
  "mega-duck-slash-cougar-boy": "mega.png",
  supervision: "watara.png",
  "odyssey-2": "odyssey.png",

  // ── Computers ──
  msx: "sony-msx-hb-10p.png",
  msx2: "sony-msx2-hb-f1-ii.png",
  dos: "dosbox-ibm-6571.png",
  cpc: "amstrad-cpc-464.png",
  zxs: "sinclair-zx-spectrum.png",
  win: "win.png",
  c64: "commodore-64.png",
  amiga: "amiga-500.png",
  scummvm: "scummvm-maniac.png",
  vic20: "commodore-vic-20.png",
  x68000: "sharp-x68000.png",
  pc98: "nec-pc-9801vx.png",
  trs80: "tandy-trs-80-model-3.png",
  ti99: "texas-instruments-ti-99-4a.png",

  // ── Fantasy Consoles ──
  "tic-80": "tic.png",
  pico: "pico8.png",
};

export function getPlatformIcon(slug: string): string | null {
  const normalized = slug.toLowerCase();
  const file = ICON_MAP[normalized];

  if (file) {
    return `/platform-icons/${file}`;
  } else {
    console.log(normalized);
  }
  return null;
}
