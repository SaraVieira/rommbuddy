import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SourceConfig } from "../../types";
import ProgressBar from "../ProgressBar";
import { useAppSync } from "../../App";
import { toast } from "sonner";
import { formatDateTime } from "../../utils/format";

interface SourceConnectedProps {
  source: SourceConfig;
  subtitle: string;
  onEdit: () => void;
  onReload: () => Promise<void>;
}

export default function SourceConnected({ source, subtitle, onEdit, onReload }: SourceConnectedProps) {
  const { syncing, progress: syncProgress, startSync, cancelSync } = useAppSync();

  const isSyncing = syncing && syncProgress && syncProgress.source_id === source.id;

  const handleSync = async () => {
    await startSync(source.id);
    await onReload();
  };

  const handleRemove = useCallback(async () => {
    if (!confirm("This will remove the source and all its synced ROMs from your library.")) return;
    try {
      await invoke("remove_source", { sourceId: source.id });
      toast.success("Source removed");
      await onReload();
    } catch (e) {
      toast.error(String(e));
    }
  }, [source, onReload]);

  return (
    <div>
      <h3 className="text-section font-semibold text-text-primary mb-md">{source.name}</h3>
      <div className="flex flex-col gap-sm text-body text-text-secondary">
        <span>{subtitle}</span>
        {source.last_synced_at && (
          <span className="text-accent font-mono font-semibold">
            Last synced: {formatDateTime(source.last_synced_at)}
          </span>
        )}
      </div>
      <div className="btn-row" style={{ marginTop: 16 }}>
        <button className="btn btn-secondary" onClick={onEdit}>Edit</button>
        <button className="btn btn-secondary" onClick={handleSync} disabled={syncing}>
          {isSyncing ? "Syncing..." : "Re-sync"}
        </button>
        <button className="btn btn-danger" onClick={handleRemove}>Remove</button>
      </div>

      {isSyncing && syncProgress && (
        <div className="mt-xl flex flex-col gap-md">
          <ProgressBar
            current={syncProgress.current}
            total={syncProgress.total}
            label={`Syncing: ${syncProgress.current_item}`}
          />
          <button className="btn btn-secondary btn-sm self-start" onClick={() => cancelSync(source.id)}>
            Cancel
          </button>
        </div>
      )}
    </div>
  );
}
