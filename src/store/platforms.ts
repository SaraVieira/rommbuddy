import { atom } from "jotai";
import { invoke } from "@tauri-apps/api/core";
import type { PlatformWithCount } from "../types";

/** Base atom holding the platform list */
export const platformsAtom = atom<PlatformWithCount[]>([]);

/** Write-only atom that fetches platforms from the backend and updates platformsAtom */
export const refreshPlatformsAtom = atom(null, async (_get, set) => {
  const platforms = await invoke<PlatformWithCount[]>(
    "get_platforms_with_counts",
  );
  set(platformsAtom, platforms);
});

/** Derived: number of platforms */
export const platformCountAtom = atom((get) => get(platformsAtom).length);

/** Derived: total ROM count across all platforms */
export const romCountAtom = atom((get) =>
  get(platformsAtom).reduce((sum, p) => sum + p.rom_count, 0),
);
