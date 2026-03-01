/**
 * Maps platform slugs to their pixel art console icon files.
 * Icons sourced from SamuraiCowboy's icon pack.
 *
 * Each key is a possible slug (lowercase). Multiple slugs can map
 * to the same icon file. The function tries an exact match first,
 * then falls back to partial/keyword matching.
 */
export const ICON_MAP: Record<string, string> = {
  // ── Nintendo ──
  gb: "gb.png",
  gbc: "gbc.png",
  gba: "gba.png",
  nes: "nintendo-famicom.png",
  fds: "nintendo-famicom-disk-system.png",
  snes: "snes.png",
  n64: "n64.png",
  "64dd": "n64.png",
  satellaview: "snes.png",
  nds: "nintendo-ds.png",
  "3ds": "nintendo-ds-lite.png",
  gamecube: "gc.png",
  wii: "wii.png",
  wiiu: "wiiu.png",
  switch: "switch.png",
  switch2: "switch2.png",
  dsi: "dsi.png",
  n3ds: "n3ds.png",
  vb: "nintendo-virtual-boy.png",
  pokemini: "pkm-mini.png",
  sufami: "sufami.png",

  // ── Sony ──
  psx: "sony-playstation.png",
  ps2: "sony-playstation.png",
  psp: "sony-psp.png",
  pspminis: "sony-psp.png",
  ps3: "ps3.png",
  ps4: "ps4.png",
  ps5: "ps5.png",
  psvita: "vita.png",
  pocketstation: "sony-playstation.png",

  // ── Sega ──
  genesis: "sega-genesis-us.png",
  segacd: "sega-genesis-cd-us.png",
  segacd32: "sega-genesis-cd-32x-us.png",
  segapico: "sega-pico-us.png",
  saturn: "sega-saturn-us.png",
  dreamcast: "sega-dreamcast.png",
  gamegear: "gg.png",
  mastersystem: "ms.png",
  sg1000: "sega-sg-1000.png",
  sc3000: "sega-sc-3000h.png",
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
  naomi: "sega-dreamcast.png",
  ngp: "snk-ngpc-platinum-silver.png",
  ngpc: "snk-ngpc-crystal-white.png",
  neocd: "neocd.png",
  hyperneogeo64: "snk-ngpc-carbon-black.png",

  // ── Arcade Sub-Systems ──
  model1: "model2.png",
  model2: "model2.png",
  model3: "model2.png",
  hikaru: "hikaru.png",
  typex: "typex.png",
  system16: "model2.png",
  system32: "model2.png",
  stv: "model2.png",
  zinc: "model2.png",

  // ── NEC ──
  pce: "nec-pc-engine-duo.png",
  pcecd: "nec-turbografxcd.png",
  sgfx: "nec-supergrafx.png",
  pcfx: "nec-pc-fx.png",

  // ── Atari ──
  atari2600: "2600.png",
  atari5200: "atari-5200-version-1.png",
  atari7800: "atari-7800-version-1.png",
  lynx: "atari-lynx.png",
  atarist: "atari-st-520.png",
  jaguar: "atari-jaguar.png",
  jaguarcd: "atari-jaguar.png",
  atarixegs: "atari-xlgs.png",
  atari8bit: "atari-800.png",

  // ── Bandai ──
  ws: "bandai-wonderswan.png",
  wsc: "bandai-wonderswan-color.png",
  swancrystal: "bandai-wonderswan-swan-crystal.png",

  // ── Other consoles ──
  colecovision: "colecovision.png",
  colecoadam: "colecoadam.png",
  intellivision: "mattel-intellivision.png",
  vectrex: "gce-vectrex-open.png",
  channelf: "channelf.png",
  "3do": "3do-panasonic.png",
  cdi: "cdi.png",
  megaduck: "mega.png",
  supervision: "watara.png",
  odyssey2: "odyssey.png",
  odyssey: "odyssey.png",
  openbor: "openbor.png",
  creativision: "creativision.png",
  arcadia2001: "arcadia2001.png",
  arduboy: "arduboy.png",
  astrocade: "astrocade.png",
  casioloopy: "casioloopy.png",
  casiopv1000: "casiopv1000.png",
  epochcv: "cassettevision.png",
  epochscv: "scv.png",
  gandw: "gameandwatch.png",
  gamate: "gamate.png",
  gamecom: "gamecom.png",
  gp32: "gp32.png",
  gp2x: "gp32.png",
  superacan: "superacan.png",
  uzebox: "uzebox.png",
  vsmile: "vsmile.png",
  videopacg7400: "videopac.png",

  // ── Commodore ──
  c64: "commodore-64.png",
  c128: "commodore-64.png",
  c16: "commodore-64.png",
  cplus4: "commodore-64.png",
  cpet: "commodore-pet-2001.png",
  vic20: "commodore-vic-20.png",
  amiga: "amiga-500.png",
  amigacd32: "amiga-cd32.png",
  amigacd: "amiga-500.png",
  cdtv: "amiga-500.png",

  // ── Apple ──
  appleii: "apple-ii.png",
  appleiigs: "apple-iie.png",

  // ── Computers ──
  samcoupe: "samcoupe.png",
  msx: "sony-msx-hb-10p.png",
  msx2: "sony-msx2-hb-f1-ii.png",
  msxturbo: "sony-msx2-hb-f1-ii.png",
  msx2plus: "sony-msx2-hb-f1-ii.png",
  dos: "dosbox-ibm-6571.png",
  cpc: "amstrad-cpc-464.png",
  gx4000: "gx4000.png",
  amstradpcw: "amstrad-cpc-464.png",
  zxspectrum: "sinclair-zx-spectrum.png",
  zxsnext: "sinclair-zx-spectrum.png",
  zx81: "sinclair-zx-81.png",
  zx80: "sinclair-zx-81.png",
  sinclairql: "sinclair-zx-spectrum.png",
  sharpmz: "sharp-x68000.png",
  cpm: "dec-vt-100.png",
  archimedes: "archimedes.png",
  electron: "electron.png",
  dragon: "dragon32.png",
  oric: "oric.png",
  thomson: "to8.png",
  fmtowns: "fmtmarty.png",
  fm7: "fm7.png",
  x1: "x1.png",
  pc8800: "pc98.png",
  pc6000: "pc98.png",
  win: "win.png",
  scummvm: "scummvm-maniac.png",
  x68000: "sharp-x68000.png",
  pc98: "nec-pc-9801vx.png",
  trs80: "tandy-trs-80-model-3.png",
  trs80coco: "tandy-color-computer-3.png",
  ti99: "texas-instruments-ti-99-4a.png",
  bbcmicro: "dec-vt-100.png",

  // ── Fantasy Consoles ──
  tic80: "tic.png",
  pico8: "pico8.png",
  wasm4: "wasm4.png",

  // ── Other consoles (continued) ──
  microvision: "MiltonBradleyMicroVisiona.png",
  ouya: "Ouya.png",

  // ── Missing icons (need artwork) ──
  atarivcs: "",
  evercade: "",
  gizmondo: "",
  intertonvc4000: "",
  laseractive: "",
  multivision: "",
  nuon: "",
  playdate: "",
  playdia: "",
  pokitto: "",
  xavixport: "",
};

export function getPlatformIcon(slug: string): string | null {
  const normalized = slug.toLowerCase();
  const file = ICON_MAP[normalized];

  if (file) {
    return `/platform-icons/${file}`;
  }
  return null;
}
