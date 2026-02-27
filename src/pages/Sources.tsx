import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SourceConfig } from "../types";
import { useAppToast, useAppSync } from "../App";
import LocalSourceSection from "../components/sources/LocalSourceSection";
import RommSourceSection from "../components/sources/RommSourceSection";

export default function Sources() {
  const toast = useAppToast();
  const { syncing, progress: syncProgress, startSync, cancelSync } = useAppSync();

  const [rommSource, setRommSource] = useState<SourceConfig | null>(null);
  const [rommName, setRommName] = useState("");
  const [rommUrl, setRommUrl] = useState("");
  const [rommUsername, setRommUsername] = useState("");
  const [rommPassword, setRommPassword] = useState("");

  const [localSource, setLocalSource] = useState<SourceConfig | null>(null);
  const [localPath, setLocalPath] = useState("");

  const loadSources = useCallback(async () => {
    try {
      const sources: SourceConfig[] = await invoke("get_sources");

      const romm = sources.find((s) => s.source_type === "romm");
      if (romm) {
        setRommSource(romm);
        setRommName(romm.name);
        setRommUrl(romm.url || "");
        const creds: string = await invoke("get_source_credentials", { sourceId: romm.id });
        const parsed = JSON.parse(creds);
        setRommUsername(parsed.username || "");
        setRommPassword(parsed.password || "");
      } else {
        setRommSource(null);
      }

      const local = sources.find((s) => s.source_type === "local");
      if (local) {
        setLocalSource(local);
        const creds: string = await invoke("get_source_credentials", { sourceId: local.id });
        const parsed = JSON.parse(creds);
        setLocalPath(parsed.path || "");
      } else {
        setLocalSource(null);
      }
    } catch (e) {
      console.error("Failed to load sources:", e);
    }
  }, []);

  useEffect(() => {
    loadSources();
  }, [loadSources]);

  return (
    <div className="page">
      <div className="flex flex-col gap-xs mb-xl">
        <h1 className="font-display text-page-title font-bold text-text-primary uppercase">Sources</h1>
        <span className="text-nav text-text-muted">Manage your ROM sources and sync connections.</span>
      </div>

      <LocalSourceSection
        source={localSource}
        initialPath={localPath}
        syncing={syncing}
        syncProgress={syncProgress}
        onStartSync={startSync}
        onCancelSync={cancelSync}
        onReload={loadSources}
        toast={toast}
      />

      <RommSourceSection
        source={rommSource}
        initialName={rommName}
        initialUrl={rommUrl}
        initialUsername={rommUsername}
        initialPassword={rommPassword}
        syncing={syncing}
        syncProgress={syncProgress}
        onStartSync={startSync}
        onCancelSync={cancelSync}
        onReload={loadSources}
        toast={toast}
      />
    </div>
  );
}
