import { atom } from "jotai";

/** Current search input (raw, pre-debounce) */
export const searchInputAtom = atom("");

/** Debounced search query sent to the backend */
export const searchAtom = atom("");

/** Selected platform ID filter */
export const selectedPlatformAtom = atom<number | null>(null);

/** Display mode */
export const viewAtom = atom<"grid" | "list">("grid");

/** Pagination offset */
export const offsetAtom = atom(0);

/** Show only favorites */
export const favoritesOnlyAtom = atom(false);

/** Maps romId -> true when saves have been detected */
export const romSavesAtom = atom<Record<number, boolean>>({});
