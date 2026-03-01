import { atom } from "jotai";
import type { SyncState } from "../hooks/useSyncState";
import type { EnrichState } from "../hooks/useEnrichState";

export const syncStateAtom = atom<SyncState>({
  syncing: false,
  progress: null,
  startSync: async () => {},
  cancelSync: async () => {},
});

export const enrichStateAtom = atom<EnrichState>({
  enriching: false,
  progress: null,
  startEnrich: async () => {},
  cancelEnrich: async () => {},
});
