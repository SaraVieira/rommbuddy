import { useState, useCallback, useRef } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import type { ScanProgress } from "../types";

export interface EnrichState {
  enriching: boolean;
  progress: ScanProgress | null;
  startEnrich: (platformId: number | null, search: string | null) => Promise<void>;
  cancelEnrich: () => Promise<void>;
}

export function useEnrichState(
  toast: (message: string, type?: "success" | "error" | "info") => void,
  onComplete?: () => void,
): EnrichState {
  const [enriching, setEnriching] = useState(false);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const enrichingRef = useRef(false);

  const startEnrich = useCallback(
    async (platformId: number | null, search: string | null) => {
      if (enrichingRef.current) return;
      enrichingRef.current = true;
      setEnriching(true);
      setProgress(null);
      try {
        const hasDb: boolean = await invoke("has_launchbox_db");
        if (!hasDb) {
          const dlChannel = new Channel<ScanProgress>();
          dlChannel.onmessage = (p) => setProgress(p);
          await invoke("update_launchbox_db", { channel: dlChannel });
        }

        const channel = new Channel<ScanProgress>();
        channel.onmessage = (p) => setProgress(p);
        await invoke("fetch_metadata", {
          platformId,
          search: search || null,
          channel,
        });
        toast("Metadata enrichment complete!", "success");
        onComplete?.();
      } catch (e) {
        toast(`Metadata enrichment failed: ${e}`, "error");
      } finally {
        enrichingRef.current = false;
        setEnriching(false);
        setProgress(null);
      }
    },
    [toast, onComplete],
  );

  const cancelEnrich = useCallback(async () => {
    await invoke("cancel_metadata");
  }, []);

  return { enriching, progress, startEnrich, cancelEnrich };
}
