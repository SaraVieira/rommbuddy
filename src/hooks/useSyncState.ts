import { useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAsyncOperation, createProgressChannel } from "./useAsyncOperation";
import type { ScanProgress } from "../types";

export interface SyncState {
  syncing: boolean;
  progress: ScanProgress | null;
  startSync: (sourceId: number) => Promise<void>;
  cancelSync: (sourceId: number) => Promise<void>;
}

export function useSyncState(onComplete?: () => void): SyncState {
  const config = useMemo(
    () => ({
      run: async (setProgress: (p: ScanProgress) => void, sourceId: number) => {
        const channel = createProgressChannel(setProgress);
        await invoke("sync_source", { sourceId, channel });
      },
      cancel: async (sourceId: number) => {
        await invoke("cancel_sync", { sourceId });
      },
      successMessage: "Sync complete!",
      errorPrefix: "Sync failed",
      onComplete,
    }),
    [onComplete],
  );

  const op = useAsyncOperation(config);

  return {
    syncing: op.running,
    progress: op.progress,
    startSync: op.start,
    cancelSync: op.cancel,
  };
}
