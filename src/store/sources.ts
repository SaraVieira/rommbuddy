import { SourceConfig } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { atom } from "jotai";

export const rommSourceAtom = atom<SourceConfig | null>(null);
export const rommNameAtom = atom("");
export const rommUrlAtom = atom("");
export const rommUsernameAtom = atom("");
export const rommPasswordAtom = atom("");
export const localSourceAtom = atom<SourceConfig | null>(null);
export const localPathAtom = atom("");

export const loadSourcesAtom = atom(null, async (_get, set) => {
  try {
    const sources: SourceConfig[] = await invoke("get_sources");

    const romm = sources.find((s) => s.source_type === "romm");
    if (romm) {
      set(rommSourceAtom, romm);
      set(rommNameAtom, romm.name);
      set(rommUrlAtom, romm.url || "");
      const creds: string = await invoke("get_source_credentials", {
        sourceId: romm.id,
      });
      const parsed = JSON.parse(creds);
      set(rommUsernameAtom, parsed.username || "");
      set(rommPasswordAtom, parsed.password || "");
    } else {
      set(rommSourceAtom, null);
    }

    const local = sources.find((s) => s.source_type === "local");
    if (local) {
      set(localSourceAtom, local);
      const creds: string = await invoke("get_source_credentials", {
        sourceId: local.id,
      });
      const parsed = JSON.parse(creds);
      set(localPathAtom, parsed.path || "");
    } else {
      set(localSourceAtom, null);
    }
  } catch (e) {
    console.error("Failed to load sources:", e);
  }
});
