import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { SourceConfig, ConnectionTestResult } from "../../types";
import { useAtomValue } from "jotai";
import { localPathAtom, localSourceAtom } from "@/store/sources";
import { useAppSync } from "@/App";
import { toast } from "sonner";
import SourceConnected from "./SourceConnected";

interface Props {
  onReload: () => Promise<void>;
}

export default function LocalSourceSection({ onReload }: Props) {
  const { startSync } = useAppSync();
  const source = useAtomValue(localSourceAtom);
  const initialPath = useAtomValue(localPathAtom);
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(source?.name ?? "");
  const [path, setPath] = useState(initialPath);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<ConnectionTestResult | null>(null);
  const [testError, setTestError] = useState<string | null>(null);

  const isFormReady = path.trim();

  const handleBrowse = async () => {
    const selected = await open({ directory: true, title: "Select ROM folder" });
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
      const result: ConnectionTestResult = await invoke("test_local_path", { path });
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
        await invoke("update_source", { sourceId: source.id, name: sourceName, url: null, credentialsJson: credsJson });
        toast.success("Source updated");
      } else if (!source) {
        await invoke("add_source", { name: sourceName, sourceType: "local", url: null, credentialsJson: credsJson });
        toast.success("Source added");
      }
      setEditing(false);
      await onReload();
      const sources: SourceConfig[] = await invoke("get_sources");
      const local = sources.find((s) => s.source_type === "local");
      if (local) await startSync(local.id);
      await onReload();
    } catch (e) {
      toast.error(String(e));
    }
  };

  return (
    <div className="card mt-3xl">
      <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">
        // Local folder
      </h2>
      <p className="text-text-secondary text-body mb-xl">
        Scan a folder on your computer or mounted SD card for ROMs.
      </p>

      {source && !editing ? (
        <SourceConnected
          source={source}
          subtitle={path}
          onEdit={() => setEditing(true)}
          onReload={onReload}
        />
      ) : (
        <>
          <div className="form-group">
            <label>Name (optional)</label>
            <input type="text" placeholder="My ROMs" value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="form-group">
            <label>ROM Folder</label>
            <div className="flex gap-md">
              <input type="text" className="flex-1 cursor-default" placeholder="/path/to/roms" value={path} onChange={(e) => setPath(e.target.value)} readOnly />
              <button className="btn btn-secondary" onClick={handleBrowse}>Browse...</button>
            </div>
          </div>

          {testResult && (
            <div className="text-body p-md bg-accent-tint-10 border border-border-accent-tint rounded-none mb-lg">
              Found {testResult.platform_count} platforms, {testResult.rom_count} ROMs
            </div>
          )}
          {testError && <div className="error-message">{testError}</div>}

          <div className="btn-row">
            <button className="btn btn-secondary" onClick={handleTest} disabled={!isFormReady || testing}>
              {testing ? "Scanning..." : "Scan Folder"}
            </button>
            <button className="btn btn-primary" onClick={handleSave} disabled={!testResult}>
              Save & Sync
            </button>
            {editing && (
              <button className="btn btn-secondary" onClick={() => setEditing(false)}>Cancel</button>
            )}
          </div>
        </>
      )}
    </div>
  );
}
