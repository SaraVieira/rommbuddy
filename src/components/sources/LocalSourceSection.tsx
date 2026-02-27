import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  SourceConfig,
  ConnectionTestResult,
  ScanProgress,
} from "../../types";
import ProgressBar from "../ProgressBar";

interface Props {
  source: SourceConfig | null;
  initialPath: string;
  syncing: boolean;
  syncProgress: ScanProgress | null;
  onStartSync: (sourceId: number) => Promise<void>;
  onCancelSync: (sourceId: number) => Promise<void>;
  onReload: () => Promise<void>;
  toast: (message: string, type?: "success" | "error" | "info") => void;
}

export default function LocalSourceSection({
  source,
  initialPath,
  syncing,
  syncProgress,
  onStartSync,
  onCancelSync,
  onReload,
  toast,
}: Props) {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(source?.name ?? "");
  const [path, setPath] = useState(initialPath);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<ConnectionTestResult | null>(
    null,
  );
  const [testError, setTestError] = useState<string | null>(null);

  const isSyncing =
    syncing && syncProgress && source && syncProgress.source_id === source.id;
  const isFormReady = path.trim();

  const handleBrowse = async () => {
    const selected = await open({
      directory: true,
      title: "Select ROM folder",
    });
    if (selected) {
      setPath(selected);
      setTestResult(null);
      setTestError(null);
    }
  };

  const handleTest = async () => {
    setTesting(true);
    setTestResult(null);
    setTestError(null);
    try {
      const result: ConnectionTestResult = await invoke("test_local_path", {
        path,
      });
      setTestResult(result);
    } catch (e) {
      setTestError(String(e));
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    const credsJson = JSON.stringify({ path });
    const sourceName = name || path.split("/").pop() || "Local ROMs";
    try {
      if (source && editing) {
        await invoke("update_source", {
          sourceId: source.id,
          name: sourceName,
          url: null,
          credentialsJson: credsJson,
        });
        toast("Source updated", "success");
      } else if (!source) {
        await invoke("add_source", {
          name: sourceName,
          sourceType: "local",
          url: null,
          credentialsJson: credsJson,
        });
        toast("Source added", "success");
      }
      setEditing(false);
      await onReload();
      const sources: SourceConfig[] = await invoke("get_sources");
      const local = sources.find((s) => s.source_type === "local");
      if (local) await onStartSync(local.id);
      await onReload();
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const handleSync = async () => {
    if (!source) return;
    await onStartSync(source.id);
    await onReload();
  };

  const handleRemove = useCallback(async () => {
    if (!source) return;
    if (
      !confirm(
        "This will remove the source and all its synced ROMs from your library.",
      )
    )
      return;
    try {
      await invoke("remove_source", { sourceId: source.id });
      toast("Source removed", "success");
      await onReload();
    } catch (e) {
      toast(String(e), "error");
    }
  }, [source, toast, onReload]);

  return (
    <div className="card mt-3xl">
      <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">
        // Local Folder
      </h2>
      <p className="text-text-secondary text-body mb-xl">
        Scan a folder on your computer or mounted SD card for ROMs.
      </p>

      {source && !editing ? (
        <div>
          <h3 className="text-section font-semibold text-text-primary mb-md">
            {source.name}
          </h3>
          <div className="flex flex-col gap-sm text-body text-text-secondary">
            <span>{path}</span>
            {source.last_synced_at && (
              <span className="text-accent font-mono font-semibold">
                Last synced: {new Date(source.last_synced_at).toLocaleString()}
              </span>
            )}
          </div>
          <div className="btn-row" style={{ marginTop: 16 }}>
            <button
              className="btn btn-secondary"
              onClick={() => setEditing(true)}
            >
              Edit
            </button>
            <button
              className="btn btn-secondary"
              onClick={handleSync}
              disabled={syncing}
            >
              {isSyncing ? "Syncing..." : "Re-sync"}
            </button>
            <button className="btn btn-danger" onClick={handleRemove}>
              Remove
            </button>
          </div>
        </div>
      ) : (
        <>
          <div className="form-group">
            <label>Name (optional)</label>
            <input
              type="text"
              placeholder="My ROMs"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>ROM Folder</label>
            <div className="flex gap-md">
              <input
                type="text"
                className="flex-1 cursor-default"
                placeholder="/path/to/roms"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                readOnly
              />
              <button className="btn btn-secondary" onClick={handleBrowse}>
                Browse...
              </button>
            </div>
          </div>

          {testResult && (
            <div className="text-body p-md bg-accent-tint-10 border border-border-accent-tint rounded-none mb-lg">
              Found {testResult.platform_count} platforms,{" "}
              {testResult.rom_count} ROMs
            </div>
          )}
          {testError && <div className="error-message">{testError}</div>}

          <div className="btn-row">
            <button
              className="btn btn-secondary"
              onClick={handleTest}
              disabled={!isFormReady || testing}
            >
              {testing ? "Scanning..." : "Scan Folder"}
            </button>
            <button
              className="btn btn-primary"
              onClick={handleSave}
              disabled={!testResult}
            >
              Save & Sync
            </button>
            {editing && (
              <button
                className="btn btn-secondary"
                onClick={() => setEditing(false)}
              >
                Cancel
              </button>
            )}
          </div>
        </>
      )}

      {isSyncing && syncProgress && (
        <div className="mt-xl flex flex-col gap-md">
          <ProgressBar
            current={syncProgress.current}
            total={syncProgress.total}
            label={`Syncing: ${syncProgress.current_item}`}
          />
          <button
            className="btn btn-secondary btn-sm self-start"
            onClick={() => source && onCancelSync(source.id)}
          >
            Cancel
          </button>
        </div>
      )}
    </div>
  );
}
