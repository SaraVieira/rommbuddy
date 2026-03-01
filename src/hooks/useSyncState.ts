import { useState, useCallback, useRef } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { ScanProgress } from "../types";

export interface SyncState {
  syncing: boolean;
  progress: ScanProgress | null;
  startSync: (sourceId: number) => Promise<void>;
  cancelSync: (sourceId: number) => Promise<void>;
}

export function useSyncState(onComplete?: () => void): SyncState {
  const [syncing, setSyncing] = useState(false);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const syncingRef = useRef(false);

  const startSync = useCallback(
    async (sourceId: number) => {
      if (syncingRef.current) return;
      syncingRef.current = true;
      setSyncing(true);
      setProgress(null);
      try {
        const channel = new Channel<ScanProgress>();
        channel.onmessage = (p) => {
          setProgress(p);
        };
        await invoke("sync_source", { sourceId, channel });
        toast.success("Sync complete!");
        onComplete?.();
      } catch (e) {
        toast.error(`Sync failed: ${e}`);
      } finally {
        syncingRef.current = false;
        setSyncing(false);
        setProgress(null);
      }
    },
    [onComplete]
  );

  const cancelSync = useCallback(async (sourceId: number) => {
    await invoke("cancel_sync", { sourceId });
  }, []);

  return { syncing, progress, startSync, cancelSync };
}
