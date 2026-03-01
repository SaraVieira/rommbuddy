import { useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAsyncOperation, createProgressChannel } from "./useAsyncOperation";
import type { ScanProgress } from "../types";

export interface EnrichState {
  enriching: boolean;
  progress: ScanProgress | null;
  startEnrich: (platformId: number | null, search: string | null) => Promise<void>;
  cancelEnrich: () => Promise<void>;
}

export function useEnrichState(onComplete?: () => void): EnrichState {
  const config = useMemo(
    () => ({
      run: async (
        setProgress: (p: ScanProgress) => void,
        platformId: number | null,
        search: string | null,
      ) => {
        const hasDb: boolean = await invoke("has_launchbox_db");
        if (!hasDb) {
          const dlChannel = createProgressChannel(setProgress);
          await invoke("update_launchbox_db", { channel: dlChannel });
        }
        const channel = createProgressChannel(setProgress);
        await invoke("fetch_metadata", {
          platformId,
          search: search || null,
          channel,
        });
      },
      cancel: async () => {
        await invoke("cancel_metadata");
      },
      successMessage: "Metadata enrichment complete!",
      errorPrefix: "Metadata enrichment failed",
      onComplete,
    }),
    [onComplete],
  );

  const op = useAsyncOperation<[number | null, string | null], []>(config);

  return {
    enriching: op.running,
    progress: op.progress,
    startEnrich: op.start,
    cancelEnrich: op.cancel,
  };
}
