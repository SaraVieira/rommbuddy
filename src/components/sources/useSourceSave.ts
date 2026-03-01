import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { SourceConfig } from "../../types";

interface UseSourceSaveOptions {
  source: SourceConfig | null;
  editing: boolean;
  sourceType: "local" | "romm";
  getName: () => string;
  getUrl: () => string | null;
  getCredentialsJson: () => string;
  setEditing: (v: boolean) => void;
  onReload: () => Promise<void>;
  startSync: (id: number) => Promise<void>;
}

export function useSourceSave({
  source,
  editing,
  sourceType,
  getName,
  getUrl,
  getCredentialsJson,
  setEditing,
  onReload,
  startSync,
}: UseSourceSaveOptions): () => Promise<void> {
  return useCallback(async () => {
    const credentialsJson = getCredentialsJson();
    const name = getName();
    const url = getUrl();
    try {
      if (source && editing) {
        await invoke("update_source", {
          sourceId: source.id,
          name,
          url,
          credentialsJson,
        });
        toast.success("Source updated");
      } else if (!source) {
        await invoke("add_source", {
          name,
          sourceType,
          url,
          credentialsJson,
        });
        toast.success("Source added");
      }
      setEditing(false);
      await onReload();
      const sources: SourceConfig[] = await invoke("get_sources");
      const match = sources.find((s) => s.source_type === sourceType);
      if (match) await startSync(match.id);
      await onReload();
    } catch (e) {
      toast.error(String(e));
    }
  }, [source, editing, sourceType, getName, getUrl, getCredentialsJson, setEditing, onReload, startSync]);
}
