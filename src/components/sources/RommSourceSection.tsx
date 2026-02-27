import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  SourceConfig,
  ConnectionTestResult,
  ScanProgress,
} from "../../types";
import ProgressBar from "../ProgressBar";

interface Props {
  source: SourceConfig | null;
  initialName: string;
  initialUrl: string;
  initialUsername: string;
  initialPassword: string;
  syncing: boolean;
  syncProgress: ScanProgress | null;
  onStartSync: (sourceId: number) => Promise<void>;
  onCancelSync: (sourceId: number) => Promise<void>;
  onReload: () => Promise<void>;
  toast: (message: string, type?: "success" | "error" | "info") => void;
}

export default function RommSourceSection({
  source,
  initialName,
  initialUrl,
  initialUsername,
  initialPassword,
  syncing,
  syncProgress,
  onStartSync,
  onCancelSync,
  onReload,
  toast,
}: Props) {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(initialName);
  const [url, setUrl] = useState(initialUrl);
  const [username, setUsername] = useState(initialUsername);
  const [password, setPassword] = useState(initialPassword);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<ConnectionTestResult | null>(
    null,
  );
  const [testError, setTestError] = useState<string | null>(null);

  const isSyncing =
    syncing && syncProgress && source && syncProgress.source_id === source.id;
  const isFormReady = url.trim() && username.trim() && password.trim();

  const handleTest = async () => {
    setTesting(true);
    setTestResult(null);
    setTestError(null);
    try {
      const result: ConnectionTestResult = await invoke(
        "test_romm_connection",
        { url, username, password },
      );
      setTestResult(result);
    } catch (e) {
      setTestError(String(e));
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    const credsJson = JSON.stringify({ username, password });
    const sourceName = name || new URL(url).hostname;
    try {
      if (source && editing) {
        await invoke("update_source", {
          sourceId: source.id,
          name: sourceName,
          url,
          credentialsJson: credsJson,
        });
        toast("Source updated", "success");
      } else if (!source) {
        await invoke("add_source", {
          name: sourceName,
          sourceType: "romm",
          url,
          credentialsJson: credsJson,
        });
        toast("Source added", "success");
      }
      setEditing(false);
      await onReload();
      const sources: SourceConfig[] = await invoke("get_sources");
      const romm = sources.find((s) => s.source_type === "romm");
      if (romm) await onStartSync(romm.id);
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
        // ROMM Server
      </h2>
      <p className="text-text-secondary text-body mb-xl">
        Connect to a ROMM server to sync its ROM catalog.
      </p>

      {source && !editing ? (
        <div>
          <h3 className="text-section font-semibold text-text-primary mb-md">
            {source.name}
          </h3>
          <div className="flex flex-col gap-sm text-body text-text-secondary">
            <span>{source.url}</span>
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
            <label>Server Name (optional)</label>
            <input
              type="text"
              placeholder="My ROMM Server"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Server URL</label>
            <input
              type="url"
              placeholder="http://192.168.1.50:3000"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Username</label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </div>

          {testResult && (
            <div className="text-body p-md bg-accent-tint-10 border border-border-accent-tint rounded-none mb-lg">
              Connected — found {testResult.platform_count} platforms,{" "}
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
              {testing ? "Testing..." : "Test Connection"}
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
