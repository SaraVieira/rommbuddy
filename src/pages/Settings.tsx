import { useState, useEffect, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  PlatformWithCount,
  CoreInfo,
  CoreMapping,
  EmulatorDef,
  ScanProgress,
  RaCredentials,
  RaTestResult,
  IgdbCredentials,
  IgdbTestResult,
  SsCredentials,
  SsTestResult,
  DatFileInfo,
  DatDetectResult,
  VerificationStats,
  SavePathOverride,
} from "../types";
import { useAppToast } from "../App";

const DEFAULT_CORES: Record<string, string> = {
  gb: "gambatte_libretro",
  gbc: "gambatte_libretro",
  gba: "mgba_libretro",
  nes: "mesen_libretro",
  snes: "snes9x_libretro",
  n64: "mupen64plus_next_libretro",
  nds: "melonds_libretro",
  psx: "swanstation_libretro",
  genesis: "genesis_plus_gx_libretro",
  arcade: "fbneo_libretro",
};

export default function Settings() {
  const toast = useAppToast();

  const [retroarchPath, setRetroarchPath] = useState<string>("");
  const [pathValid, setPathValid] = useState(false);
  const [cores, setCores] = useState<CoreInfo[]>([]);
  const [platforms, setPlatforms] = useState<PlatformWithCount[]>([]);
  const [mappings, setMappings] = useState<CoreMapping[]>([]);
  const [availableCores, setAvailableCores] = useState<CoreInfo[]>([]);
  const [loadingAvailable, setLoadingAvailable] = useState(false);
  const [installingCore, setInstallingCore] = useState<string | null>(null);
  const [coreSearch, setCoreSearch] = useState("");

  // Emulator state
  const [emulators, setEmulators] = useState<EmulatorDef[]>([]);
  const [emulatorPaths, setEmulatorPaths] = useState<Record<string, string>>(
    {}
  );

  // Save directory overrides
  const [savePaths, setSavePaths] = useState<Record<string, SavePathOverride>>({});

  const loadSettings = useCallback(async () => {
    try {
      const path: string | null = await invoke("get_retroarch_path");
      if (path) {
        setRetroarchPath(path);
        setPathValid(true);
        const detected: CoreInfo[] = await invoke("detect_cores", {
          retroarchPath: path,
        });
        setCores(detected);
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }, []);

  const loadEmulators = useCallback(async () => {
    try {
      const emus: EmulatorDef[] = await invoke("get_emulators");
      setEmulators(emus);
      const paths: Record<string, string> = await invoke(
        "get_emulator_paths"
      );
      const detected: [string, string][] = await invoke("detect_emulators");
      // Merge: store paths take priority, then auto-detected
      const merged = { ...paths };
      for (const [id, path] of detected) {
        if (!merged[id]) {
          merged[id] = path;
          // Auto-save detected paths
          try {
            await invoke("set_emulator_path", { emulatorId: id, path });
          } catch {
            // ignore save errors for auto-detect
          }
        }
      }
      setEmulatorPaths(merged);
    } catch (e) {
      console.error("Failed to load emulators:", e);
    }
  }, []);

  const loadSavePaths = useCallback(async () => {
    try {
      const paths = await invoke<Record<string, SavePathOverride>>("get_save_paths");
      setSavePaths(paths);
    } catch (e) {
      console.error("Failed to load save paths:", e);
    }
  }, []);

  const loadMappings = useCallback(async () => {
    try {
      const m: CoreMapping[] = await invoke("get_core_mappings");
      setMappings(m);
      const p: PlatformWithCount[] = await invoke("get_platforms_with_counts");
      setPlatforms(p);
    } catch (e) {
      console.error("Failed to load mappings:", e);
    }
  }, []);

  const loadDatFiles = useCallback(async () => {
    try {
      const files: DatFileInfo[] = await invoke("get_dat_files");
      setDatFiles(files);
    } catch (e) {
      console.error("Failed to load DAT files:", e);
    }
  }, []);

  const loadRaCredentials = useCallback(async () => {
    try {
      const creds: RaCredentials | null = await invoke("get_ra_credentials");
      if (creds) {
        setRaUsername(creds.username);
        setRaApiKey(creds.api_key);
        setRaStatus("ok");
        setRaStatusMessage(`Connected as ${creds.username}`);
      }
    } catch (e) {
      console.error("Failed to load RA credentials:", e);
    }
  }, []);

  const loadIgdbCredentials = useCallback(async () => {
    try {
      const creds: IgdbCredentials | null = await invoke("get_igdb_credentials");
      if (creds) {
        setIgdbClientId(creds.client_id);
        setIgdbClientSecret(creds.client_secret);
        setIgdbStatus("ok");
        setIgdbStatusMessage("Credentials saved");
      }
    } catch (e) {
      console.error("Failed to load IGDB credentials:", e);
    }
  }, []);

  const loadSsCredentials = useCallback(async () => {
    try {
      const creds: SsCredentials | null = await invoke("get_ss_credentials");
      if (creds) {
        setSsUsername(creds.username);
        setSsPassword(creds.password);
        setSsStatus("ok");
        setSsStatusMessage(`Credentials saved for ${creds.username}`);
      }
    } catch (e) {
      console.error("Failed to load ScreenScraper credentials:", e);
    }
  }, []);

  useEffect(() => {
    loadSettings();
    loadEmulators();
    loadSavePaths();
    loadMappings();
    loadRaCredentials();
    loadIgdbCredentials();
    loadSsCredentials();
    loadDatFiles();
  }, [loadSettings, loadEmulators, loadSavePaths, loadMappings, loadRaCredentials, loadIgdbCredentials, loadSsCredentials, loadDatFiles]);

  const handleBrowse = async () => {
    const selected = await open({
      directory: false,
      multiple: false,
      title: "Select RetroArch executable",
    });
    if (selected) {
      setRetroarchPath(selected as string);
      await handleSavePath(selected as string);
    }
  };

  const handleSavePath = async (path: string) => {
    try {
      await invoke("set_retroarch_path", { path });
      setPathValid(true);
      toast("RetroArch path saved", "success");
      const detected: CoreInfo[] = await invoke("detect_cores", {
        retroarchPath: path,
      });
      setCores(detected);
      // Auto-apply default mappings
      for (const platform of platforms) {
        const defaultCore = DEFAULT_CORES[platform.slug];
        if (defaultCore) {
          const found = detected.find((c) => c.core_name === defaultCore);
          if (found && !mappings.find((m) => m.platform_id === platform.id)) {
            await invoke("set_core_mapping", {
              platformId: platform.id,
              coreName: found.core_name,
              corePath: found.core_path,
              emulatorType: "retroarch",
            });
          }
        }
      }
      await loadMappings();
    } catch (e) {
      setPathValid(false);
      toast(String(e), "error");
    }
  };

  const handleEmulatorBrowse = async (emulatorId: string) => {
    const selected = await open({
      directory: false,
      multiple: false,
      title: `Select ${emulatorId} application`,
    });
    if (selected) {
      try {
        await invoke("set_emulator_path", {
          emulatorId,
          path: selected as string,
        });
        setEmulatorPaths((prev) => ({
          ...prev,
          [emulatorId]: selected as string,
        }));
        toast("Emulator path saved", "success");
      } catch (e) {
        toast(String(e), "error");
      }
    }
  };

  const handleSaveDirBrowse = async (emulatorId: string, dirType: "save_dir" | "state_dir") => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: `Select ${dirType === "save_dir" ? "save files" : "save states"} directory for ${emulatorId}`,
    });
    if (selected) {
      const existing = savePaths[emulatorId] || { save_dir: null, state_dir: null };
      const updated = { ...existing, [dirType]: selected as string };
      try {
        await invoke("set_save_path", { emulatorId, saveDir: updated.save_dir, stateDir: updated.state_dir });
        setSavePaths((prev) => ({ ...prev, [emulatorId]: updated }));
        toast("Save directory saved", "success");
      } catch (e) {
        toast(String(e), "error");
      }
    }
  };

  const handleResetSavePath = async (emulatorId: string) => {
    try {
      await invoke("set_save_path", { emulatorId, saveDir: null, stateDir: null });
      setSavePaths((prev) => { const next = { ...prev }; delete next[emulatorId]; return next; });
      toast("Reset to default save directories", "success");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const handleCoreChange = async (
    platformId: number,
    value: string,
    _platformSlug: string
  ) => {
    // Value format: "retroarch:{coreName}" or "emu:{emulatorId}"
    if (value.startsWith("emu:")) {
      const emulatorId = value.slice(4);
      try {
        await invoke("set_core_mapping", {
          platformId,
          coreName: emulatorId,
          corePath: "",
          emulatorType: emulatorId,
        });
        toast("Emulator mapping saved", "success");
        await loadMappings();
      } catch (e) {
        toast(String(e), "error");
      }
    } else if (value.startsWith("retroarch:")) {
      const coreName = value.slice(10);
      const core = cores.find((c) => c.core_name === coreName);
      if (!core) return;
      try {
        await invoke("set_core_mapping", {
          platformId,
          coreName: core.core_name,
          corePath: core.core_path,
          emulatorType: "retroarch",
        });
        toast("Core mapping saved", "success");
        await loadMappings();
      } catch (e) {
        toast(String(e), "error");
      }
    }
  };

  const getMappingForPlatform = (platformId: number) =>
    mappings.find((m) => m.platform_id === platformId);

  const getMappingValue = (mapping: CoreMapping | undefined): string => {
    if (!mapping) return "";
    if (mapping.emulator_type !== "retroarch") {
      return `emu:${mapping.emulator_type}`;
    }
    return `retroarch:${mapping.core_name}`;
  };

  const getEmulatorsForPlatform = (platformSlug: string): EmulatorDef[] => {
    return emulators.filter(
      (e) => e.platforms.includes(platformSlug) && emulatorPaths[e.id]
    );
  };

  const handleLoadAvailable = async () => {
    setLoadingAvailable(true);
    try {
      const available: CoreInfo[] = await invoke("get_available_cores", {
        retroarchPath,
      });
      setAvailableCores(available);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setLoadingAvailable(false);
    }
  };

  const handleInstallCore = async (coreName: string) => {
    setInstallingCore(coreName);
    try {
      await invoke("install_core", {
        retroarchPath,
        coreName,
      });
      toast(`Installed ${coreName}`, "success");
      // Refresh cores and available list
      const detected: CoreInfo[] = await invoke("detect_cores", {
        retroarchPath,
      });
      setCores(detected);
      setAvailableCores((prev) =>
        prev.filter((c) => c.core_name !== coreName)
      );
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setInstallingCore(null);
    }
  };

  const filteredAvailableCores = availableCores.filter((core) => {
    const label = core.display_name || core.core_name;
    return label.toLowerCase().includes(coreSearch.toLowerCase());
  });

  // Tab state
  const [activeTab, setActiveTab] = useState<"retroarch" | "emulators" | "integrations" | "dat">("retroarch");

  // Platform selector dialog state (for DAT import)
  const [showPlatformDialog, setShowPlatformDialog] = useState(false);
  const [pendingDatPath, setPendingDatPath] = useState<string | null>(null);
  const [pendingDatHeaderName, setPendingDatHeaderName] = useState("");
  const [platformSearch, setPlatformSearch] = useState("");

  // Metadata DB state
  const [updatingMetadataDb, setUpdatingMetadataDb] = useState(false);
  const [metadataDbProgress, setMetadataDbProgress] = useState<ScanProgress | null>(null);

  // DAT files state
  const [datFiles, setDatFiles] = useState<DatFileInfo[]>([]);
  const [importingDat, setImportingDat] = useState(false);
  const [datProgress, setDatProgress] = useState<ScanProgress | null>(null);
  const [verifying, setVerifying] = useState(false);
  const [verifyProgress, setVerifyProgress] = useState<ScanProgress | null>(null);

  // RetroAchievements state
  const [raUsername, setRaUsername] = useState("");
  const [raApiKey, setRaApiKey] = useState("");
  const [raStatus, setRaStatus] = useState<"unchecked" | "ok" | "error" | "testing">("unchecked");
  const [raStatusMessage, setRaStatusMessage] = useState("");

  // IGDB / Twitch state
  const [igdbClientId, setIgdbClientId] = useState("");
  const [igdbClientSecret, setIgdbClientSecret] = useState("");
  const [igdbStatus, setIgdbStatus] = useState<"unchecked" | "ok" | "error" | "testing">("unchecked");
  const [igdbStatusMessage, setIgdbStatusMessage] = useState("");

  // ScreenScraper state
  const [ssUsername, setSsUsername] = useState("");
  const [ssPassword, setSsPassword] = useState("");
  const [ssStatus, setSsStatus] = useState<"unchecked" | "ok" | "error" | "testing">("unchecked");
  const [ssStatusMessage, setSsStatusMessage] = useState("");

  const handleUpdateMetadataDb = useCallback(async () => {
    if (updatingMetadataDb) return;
    setUpdatingMetadataDb(true);
    setMetadataDbProgress(null);
    try {
      const channel = new Channel<ScanProgress>();
      channel.onmessage = (p) => setMetadataDbProgress(p);
      await invoke("update_launchbox_db", { channel });
      toast("Metadata database updated!", "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setUpdatingMetadataDb(false);
      setMetadataDbProgress(null);
    }
  }, [updatingMetadataDb, toast]);

  const handleSaveRaCredentials = async () => {
    try {
      await invoke("set_ra_credentials", { username: raUsername, apiKey: raApiKey });
      toast("RetroAchievements credentials saved", "success");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const handleTestRaConnection = async () => {
    setRaStatus("testing");
    try {
      const result: RaTestResult = await invoke("test_ra_connection", {
        username: raUsername,
        apiKey: raApiKey,
      });
      setRaStatus(result.success ? "ok" : "error");
      setRaStatusMessage(result.message);
      if (result.success) {
        await invoke("set_ra_credentials", { username: raUsername, apiKey: raApiKey });
      }
    } catch (e) {
      setRaStatus("error");
      setRaStatusMessage(String(e));
    }
  };

  const handleSaveIgdbCredentials = async () => {
    try {
      await invoke("set_igdb_credentials", { clientId: igdbClientId, clientSecret: igdbClientSecret });
      toast("IGDB credentials saved", "success");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const handleTestIgdbConnection = async () => {
    setIgdbStatus("testing");
    try {
      const result: IgdbTestResult = await invoke("test_igdb_connection", {
        clientId: igdbClientId,
        clientSecret: igdbClientSecret,
      });
      setIgdbStatus(result.success ? "ok" : "error");
      setIgdbStatusMessage(result.message);
      if (result.success) {
        await invoke("set_igdb_credentials", { clientId: igdbClientId, clientSecret: igdbClientSecret });
      }
    } catch (e) {
      setIgdbStatus("error");
      setIgdbStatusMessage(String(e));
    }
  };

  const handleSaveSsCredentials = async () => {
    try {
      await invoke("set_ss_credentials", { username: ssUsername, password: ssPassword });
      toast("ScreenScraper credentials saved", "success");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const handleTestSsConnection = async () => {
    setSsStatus("testing");
    try {
      const result: SsTestResult = await invoke("test_ss_connection", {
        username: ssUsername,
        password: ssPassword,
      });
      setSsStatus(result.success ? "ok" : "error");
      setSsStatusMessage(result.message);
      if (result.success) {
        await invoke("set_ss_credentials", { username: ssUsername, password: ssPassword });
      }
    } catch (e) {
      setSsStatus("error");
      setSsStatusMessage(String(e));
    }
  };

  const handleImportDat = async () => {
    const selected = await open({
      directory: false,
      multiple: false,
      title: "Select DAT file",
      filters: [{ name: "DAT Files", extensions: ["dat", "xml"] }],
    });
    if (!selected) return;

    try {
      // Auto-detect platform from DAT header
      const result: DatDetectResult = await invoke("detect_dat_platform", {
        filePath: selected as string,
      });

      if (!result.detected_slug) {
        // Show platform selector dialog
        setPendingDatPath(selected as string);
        setPendingDatHeaderName(result.header_name);
        setPlatformSearch("");
        setShowPlatformDialog(true);
        return;
      }

      await doImportDat(selected as string, result.detected_slug);
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const doImportDat = async (filePath: string, platformSlug: string) => {
    setImportingDat(true);
    setDatProgress(null);
    try {
      const channel = new Channel<ScanProgress>();
      channel.onmessage = (p) => setDatProgress(p);

      const fileName = filePath.toLowerCase();
      const datType = fileName.includes("redump") ? "redump" : "no-intro";

      await invoke("import_dat_file", {
        filePath,
        datType,
        platformSlug,
        channel,
      });
      toast("DAT file imported!", "success");
      loadDatFiles();
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setImportingDat(false);
      setDatProgress(null);
    }
  };

  const handlePlatformSelect = async (slug: string) => {
    setShowPlatformDialog(false);
    if (pendingDatPath) {
      await doImportDat(pendingDatPath, slug);
      setPendingDatPath(null);
    }
  };

  const handleRemoveDat = async (id: number) => {
    try {
      await invoke("remove_dat_file", { datFileId: id });
      setDatFiles((prev) => prev.filter((d) => d.id !== id));
      toast("DAT file removed", "success");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  return (
    <div className="page">
      <h1 className="font-display text-page-title font-bold text-text-primary mb-md uppercase">Settings</h1>
      <p className="text-body text-text-muted mb-xl">Configure emulators, integrations, and metadata.</p>

      <div className="flex items-center gap-0 border-b border-border mb-xl">
        {(["retroarch", "emulators", "integrations", "dat"] as const).map((tab) => (
          <button
            key={tab}
            className={`px-xl py-md text-label font-mono uppercase tracking-wide border-b-2 transition-colors ${
              activeTab === tab
                ? "border-accent text-accent"
                : "border-transparent text-text-muted hover:text-text-secondary"
            }`}
            onClick={() => setActiveTab(tab)}
          >
            {tab === "dat" ? "DAT FILES" : tab.toUpperCase()}
          </button>
        ))}
      </div>

      {activeTab === "retroarch" && (
        <>
          <section>
            <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">// RetroArch Path</h2>
            <div className="card">
              <div className="form-group">
                <label>RetroArch Path</label>
                <div className="flex gap-md">
                  <input
                    type="text"
                    className="flex-1"
                    value={retroarchPath}
                    onChange={(e) => setRetroarchPath(e.target.value)}
                    placeholder="/path/to/retroarch"
                  />
                  <button className="btn btn-secondary" onClick={handleBrowse}>
                    Browse
                  </button>
                  <button
                    className="btn btn-primary"
                    onClick={() => handleSavePath(retroarchPath)}
                    disabled={!retroarchPath}
                  >
                    Save
                  </button>
                </div>
                {retroarchPath && (
                  <span className={pathValid ? "text-accent font-mono font-semibold" : "text-error font-mono font-semibold"}>
                    {pathValid ? "[FOUND]" : "[NOT FOUND]"}
                  </span>
                )}
              </div>
            </div>
          </section>

          {platforms.length > 0 && (
            <section className="mt-3xl">
              <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">// Core Mappings</h2>
              <div className="card">
                <table className="w-full border-collapse">
                  <thead>
                    <tr>
                      <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">Platform</th>
                      <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">Emulator / Core</th>
                      <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    {platforms.map((platform) => {
                      const mapping = getMappingForPlatform(platform.id);
                      const defaultCore = DEFAULT_CORES[platform.slug];
                      const platformEmulators = getEmulatorsForPlatform(platform.slug);
                      const hasRetroarchCores = pathValid && cores.length > 0;
                      return (
                        <tr key={platform.id}>
                          <td className="p-md px-lg text-body text-text-primary border-b border-border align-middle">
                            {platform.name}{" "}
                            <span className="text-text-dim text-nav">({platform.rom_count})</span>
                          </td>
                          <td className="p-md px-lg text-body text-text-primary border-b border-border align-middle">
                            <select
                              className="w-full py-sm px-md rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-body"
                              value={getMappingValue(mapping)}
                              onChange={(e) => handleCoreChange(platform.id, e.target.value, platform.slug)}
                            >
                              <option value="">Select...</option>
                              {platformEmulators.length > 0 && (
                                <optgroup label="Standalone Emulators">
                                  {platformEmulators.map((emu) => (
                                    <option key={`emu:${emu.id}`} value={`emu:${emu.id}`}>
                                      {emu.name}
                                    </option>
                                  ))}
                                </optgroup>
                              )}
                              {hasRetroarchCores && (
                                <optgroup label="RetroArch Cores">
                                  {cores.map((core) => (
                                    <option key={`retroarch:${core.core_name}`} value={`retroarch:${core.core_name}`}>
                                      {core.display_name || core.core_name}
                                      {core.core_name === defaultCore ? " (recommended)" : ""}
                                    </option>
                                  ))}
                                </optgroup>
                              )}
                            </select>
                          </td>
                          <td className="p-md px-lg text-body text-text-primary border-b border-border align-middle">
                            {mapping ? (
                              <span className="text-accent font-mono font-semibold">[OK]</span>
                            ) : (
                              <span className="text-error font-mono font-semibold">[MISSING]</span>
                            )}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </section>
          )}

          {pathValid && (
            <section className="mt-3xl">
              <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">// Install Cores</h2>
              <div className="card">
                {availableCores.length === 0 ? (
                  <button
                    className="btn btn-secondary"
                    onClick={handleLoadAvailable}
                    disabled={loadingAvailable}
                  >
                    {loadingAvailable ? "Loading..." : "Load Available Cores"}
                  </button>
                ) : (
                  <>
                    <div className="flex items-center gap-lg mb-lg">
                      <input
                        type="text"
                        className="flex-1 px-[10px] py-[6px] rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-body focus:border-accent outline-none"
                        placeholder="Search cores..."
                        value={coreSearch}
                        onChange={(e) => setCoreSearch(e.target.value)}
                      />
                      <span className="text-text-muted text-nav whitespace-nowrap">
                        {filteredAvailableCores.length} cores available
                      </span>
                    </div>
                    <div className="max-h-[400px] overflow-y-auto flex flex-col gap-xs">
                      {filteredAvailableCores.map((core) => (
                        <div key={core.core_name} className="flex items-center justify-between py-[6px] px-lg rounded-none hover:bg-bg-elevated">
                          <span className="text-body text-text-primary">
                            {core.display_name || core.core_name}
                          </span>
                          <button
                            className="btn btn-primary btn-sm"
                            onClick={() => handleInstallCore(core.core_name)}
                            disabled={installingCore !== null}
                          >
                            {installingCore === core.core_name ? "Installing..." : "Install"}
                          </button>
                        </div>
                      ))}
                    </div>
                  </>
                )}
              </div>
            </section>
          )}
        </>
      )}

      {activeTab === "emulators" && (
        <section>
          <p className="text-body text-text-muted mb-xl">
            Configure paths to standalone emulators. These can be used instead of RetroArch cores for specific platforms.
          </p>
          <div className="card">
            {emulators.map((emu) => (
              <div key={emu.id} className="form-group">
                <label>{emu.name}</label>
                <div className="flex gap-md">
                  <input
                    type="text"
                    className="flex-1"
                    value={emulatorPaths[emu.id] || ""}
                    readOnly
                    placeholder={`/Applications/${emu.name}.app`}
                  />
                  <button
                    className="btn btn-secondary"
                    onClick={() => handleEmulatorBrowse(emu.id)}
                  >
                    Browse
                  </button>
                </div>
                <span
                  className={
                    emulatorPaths[emu.id] ? "text-accent font-mono font-semibold" : "text-text-muted font-mono font-semibold"
                  }
                >
                  {emulatorPaths[emu.id] ? "[FOUND]" : "[NOT FOUND]"}
                </span>
              </div>
            ))}
          </div>

          <div className="mt-3xl">
            <h2 className="font-mono text-[13px] font-semibold text-accent uppercase tracking-wide mb-lg">// SAVE DIRECTORIES</h2>
            <div className="bg-bg-card border border-border p-3xl flex flex-col gap-xl">
              <p className="font-mono text-[12px] text-text-muted leading-[1.6]">
                Override the default save file and save state directories for each emulator. Leave blank to use each emulator's default paths.
              </p>

              {["retroarch", ...emulators.map((e) => e.id)].map((emuId) => {
                const override = savePaths[emuId];
                const hasCustom = override && (override.save_dir || override.state_dir);
                return (
                  <div key={emuId} className="flex flex-col gap-md border border-border p-xl">
                    <div className="flex items-center justify-between">
                      <span className="font-mono text-[12px] font-semibold text-text-primary uppercase tracking-wide">
                        {emuId}
                      </span>
                      {hasCustom ? (
                        <div className="flex items-center gap-md">
                          <span className="font-mono text-[10px] font-semibold text-accent">
                            [CUSTOM]
                          </span>
                          <button
                            className="btn btn-sm btn-secondary"
                            onClick={() => handleResetSavePath(emuId)}
                          >
                            RESET
                          </button>
                        </div>
                      ) : (
                        <span className="font-mono text-[10px] font-semibold text-text-muted">
                          [USING DEFAULTS]
                        </span>
                      )}
                    </div>

                    <div className="flex flex-col gap-sm">
                      <span className="font-mono text-[10px] font-semibold text-text-muted uppercase">
                        SAVES
                      </span>
                      <div className="flex gap-md">
                        <input
                          type="text"
                          className="flex-1 bg-bg-elevated border border-border font-mono text-[11px] text-text-primary px-lg py-sm"
                          value={override?.save_dir || ""}
                          readOnly
                          placeholder="Default save directory"
                        />
                        <button
                          className="btn btn-secondary btn-sm"
                          onClick={() => handleSaveDirBrowse(emuId, "save_dir")}
                        >
                          BROWSE
                        </button>
                      </div>
                    </div>

                    <div className="flex flex-col gap-sm">
                      <span className="font-mono text-[10px] font-semibold text-text-muted uppercase">
                        STATES
                      </span>
                      <div className="flex gap-md">
                        <input
                          type="text"
                          className="flex-1 bg-bg-elevated border border-border font-mono text-[11px] text-text-primary px-lg py-sm"
                          value={override?.state_dir || ""}
                          readOnly
                          placeholder="Default save states directory"
                        />
                        <button
                          className="btn btn-secondary btn-sm"
                          onClick={() => handleSaveDirBrowse(emuId, "state_dir")}
                        >
                          BROWSE
                        </button>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </section>
      )}

      {activeTab === "integrations" && (
        <>
          <section>
            <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">// Metadata Database</h2>
            <div className="card">
              <p className="text-body text-text-muted mb-lg">
                LaunchBox metadata database provides game descriptions, ratings, release dates, and cover art. Update periodically to get the latest metadata.
              </p>
              <button
                className="btn btn-secondary"
                disabled={updatingMetadataDb}
                onClick={handleUpdateMetadataDb}
              >
                {updatingMetadataDb ? "Updating..." : "Update Metadata DB"}
              </button>
              {metadataDbProgress && (
                <div className="mt-md text-body text-text-muted">
                  <div className="flex items-center justify-between">
                    <span className="truncate mr-md">{metadataDbProgress.current_item}</span>
                    {metadataDbProgress.total > 0 && (
                      <span className="shrink-0">
                        {metadataDbProgress.current} / {metadataDbProgress.total}
                      </span>
                    )}
                  </div>
                  {metadataDbProgress.total > 1 && (
                    <div className="mt-xs h-[3px] bg-border">
                      <div
                        className="h-full bg-accent transition-[width] duration-150"
                        style={{ width: `${(metadataDbProgress.current / metadataDbProgress.total) * 100}%` }}
                      />
                    </div>
                  )}
                </div>
              )}
            </div>
          </section>

          <section className="mt-3xl">
            <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">// RetroAchievements</h2>
            <div className="card">
              <p className="text-body text-text-muted mb-xl">
                Connect your RetroAchievements account to view achievement progress for your games. Get your Web API Key from retroachievements.org/controlpanel.php
              </p>
              <div className="flex flex-col gap-lg">
                <div className="form-group">
                  <label>Username</label>
                  <input
                    type="text"
                    value={raUsername}
                    onChange={(e) => setRaUsername(e.target.value)}
                    placeholder="YourUsername"
                  />
                </div>
                <div className="form-group">
                  <label>Web API Key</label>
                  <input
                    type="password"
                    value={raApiKey}
                    onChange={(e) => setRaApiKey(e.target.value)}
                    placeholder="Your API key"
                  />
                </div>
                <div className="flex items-center gap-md">
                  <button className="btn btn-primary" onClick={handleSaveRaCredentials}>
                    Save Credentials
                  </button>
                  <button
                    className="btn btn-secondary"
                    onClick={handleTestRaConnection}
                    disabled={raStatus === "testing" || !raUsername || !raApiKey}
                  >
                    {raStatus === "testing" ? "Testing..." : "Test Connection"}
                  </button>
                  {raStatus !== "unchecked" && raStatus !== "testing" && (
                    <span className={`text-body font-mono font-semibold ${raStatus === "ok" ? "text-accent" : "text-error"}`}>
                      [{raStatus === "ok" ? "OK" : "ERROR"}] {raStatusMessage}
                    </span>
                  )}
                </div>
              </div>
            </div>
          </section>

          <section className="mt-3xl">
            <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">// IGDB / Twitch</h2>
            <div className="card">
              <p className="text-body text-text-muted mb-xl">
                Connect to IGDB for richer game metadata including descriptions, screenshots, themes, and ratings. Create a Twitch developer application at dev.twitch.tv/console to get your Client ID and Secret.
              </p>
              <div className="flex flex-col gap-lg">
                <div className="form-group">
                  <label>Client ID</label>
                  <input
                    type="text"
                    value={igdbClientId}
                    onChange={(e) => setIgdbClientId(e.target.value)}
                    placeholder="Your Twitch Client ID"
                  />
                </div>
                <div className="form-group">
                  <label>Client Secret</label>
                  <input
                    type="password"
                    value={igdbClientSecret}
                    onChange={(e) => setIgdbClientSecret(e.target.value)}
                    placeholder="Your Twitch Client Secret"
                  />
                </div>
                <div className="flex items-center gap-md">
                  <button className="btn btn-primary" onClick={handleSaveIgdbCredentials}>
                    Save Credentials
                  </button>
                  <button
                    className="btn btn-secondary"
                    onClick={handleTestIgdbConnection}
                    disabled={igdbStatus === "testing" || !igdbClientId || !igdbClientSecret}
                  >
                    {igdbStatus === "testing" ? "Testing..." : "Test Connection"}
                  </button>
                  {igdbStatus !== "unchecked" && igdbStatus !== "testing" && (
                    <span className={`text-body font-mono font-semibold ${igdbStatus === "ok" ? "text-accent" : "text-error"}`}>
                      [{igdbStatus === "ok" ? "OK" : "ERROR"}] {igdbStatusMessage}
                    </span>
                  )}
                </div>
              </div>
            </div>
          </section>

          <section className="mt-3xl">
            <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">// ScreenScraper</h2>
            <div className="card">
              <p className="text-body text-text-muted mb-xl">
                Connect to ScreenScraper for high-quality retro game artwork and metadata including box art, screenshots, and detailed game info. Create a free account at screenscraper.fr for higher rate limits.
              </p>
              <div className="flex flex-col gap-lg">
                <div className="form-group">
                  <label>Username</label>
                  <input
                    type="text"
                    value={ssUsername}
                    onChange={(e) => setSsUsername(e.target.value)}
                    placeholder="Your ScreenScraper username"
                  />
                </div>
                <div className="form-group">
                  <label>Password</label>
                  <input
                    type="password"
                    value={ssPassword}
                    onChange={(e) => setSsPassword(e.target.value)}
                    placeholder="Your ScreenScraper password"
                  />
                </div>
                <div className="flex items-center gap-md">
                  <button className="btn btn-primary" onClick={handleSaveSsCredentials}>
                    Save Credentials
                  </button>
                  <button
                    className="btn btn-secondary"
                    onClick={handleTestSsConnection}
                    disabled={ssStatus === "testing" || !ssUsername || !ssPassword}
                  >
                    {ssStatus === "testing" ? "Testing..." : "Test Connection"}
                  </button>
                  {ssStatus !== "unchecked" && ssStatus !== "testing" && (
                    <span className={`text-body font-mono font-semibold ${ssStatus === "ok" ? "text-accent" : "text-error"}`}>
                      [{ssStatus === "ok" ? "OK" : "ERROR"}] {ssStatusMessage}
                    </span>
                  )}
                </div>
              </div>
            </div>
          </section>
        </>
      )}

      {activeTab === "dat" && (
        <>
          <section>
            <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">// DAT File Management</h2>
            <div className="card">
              <p className="text-body text-text-muted mb-lg">
                Import No-Intro and Redump DAT files to verify your ROMs against known-good databases. When a platform cannot be auto-detected, you will be asked to select it manually.
              </p>
              <div className="flex items-center gap-lg mb-lg">
                <button
                  className="btn btn-primary"
                  disabled={importingDat}
                  onClick={handleImportDat}
                >
                  {importingDat ? "Importing..." : "Import DAT File"}
                </button>
                <span className="text-text-muted text-nav">{datFiles.length} DAT file{datFiles.length !== 1 ? "s" : ""} imported</span>
              </div>
              {datProgress && (
                <div className="mb-lg text-body text-text-muted">
                  <div className="flex items-center justify-between">
                    <span className="truncate mr-md">{datProgress.current_item}</span>
                    {datProgress.total > 0 && (
                      <span className="shrink-0">
                        {datProgress.current} / {datProgress.total}
                      </span>
                    )}
                  </div>
                  {datProgress.total > 1 && (
                    <div className="mt-xs h-[3px] bg-border">
                      <div
                        className="h-full bg-accent transition-[width] duration-150"
                        style={{ width: `${(datProgress.current / datProgress.total) * 100}%` }}
                      />
                    </div>
                  )}
                </div>
              )}
              {datFiles.length > 0 && (
                <table className="w-full border-collapse">
                  <thead>
                    <tr>
                      <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">Name</th>
                      <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">Platform</th>
                      <th className="text-right p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">Entries</th>
                      <th className="p-md px-lg border-b border-border" style={{ width: 60 }}></th>
                    </tr>
                  </thead>
                  <tbody>
                    {datFiles.map((dat) => (
                      <tr key={dat.id}>
                        <td className="p-md px-lg text-body text-text-primary border-b border-border">
                          {dat.name}
                        </td>
                        <td className="p-md px-lg text-body text-text-muted border-b border-border font-mono">
                          {dat.platform_slug}
                        </td>
                        <td className="p-md px-lg text-body text-text-muted border-b border-border text-right font-mono">
                          {dat.entry_count.toLocaleString()}
                        </td>
                        <td className="p-md px-lg border-b border-border text-right">
                          <button
                            className="text-error font-mono text-badge hover:underline cursor-pointer bg-transparent border-none"
                            onClick={() => handleRemoveDat(dat.id)}
                          >
                            REMOVE
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          </section>

          <section className="mt-3xl">
            <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">// Verify Library</h2>
            <div className="card">
              <p className="text-body text-text-muted mb-lg">
                Compare ROM hashes against imported DAT files to verify integrity. Verified ROMs show a green badge, bad dumps show a warning.
              </p>
              <button
                className="btn btn-secondary"
                disabled={datFiles.length === 0 || verifying}
                onClick={async () => {
                  setVerifying(true);
                  setVerifyProgress(null);
                  try {
                    const channel = new Channel<ScanProgress>();
                    channel.onmessage = (p) => setVerifyProgress(p);
                    const stats: VerificationStats = await invoke("verify_library", {
                      platformId: null,
                      channel,
                    });
                    toast(
                      `Verified ${stats.verified}, Unverified ${stats.unverified}, Bad Dumps ${stats.bad_dump}`,
                      "success",
                    );
                  } catch (e) {
                    toast(String(e), "error");
                  } finally {
                    setVerifying(false);
                    setVerifyProgress(null);
                  }
                }}
              >
                {verifying ? "Verifying..." : "Verify Library"}
              </button>
              {verifyProgress && (
                <div className="mt-md text-body text-text-muted">
                  <div className="flex items-center justify-between">
                    <span className="truncate mr-md">{verifyProgress.current_item}</span>
                    {verifyProgress.total > 0 && (
                      <span className="shrink-0">
                        {verifyProgress.current} / {verifyProgress.total}
                      </span>
                    )}
                  </div>
                  {verifyProgress.total > 1 && (
                    <div className="mt-xs h-[3px] bg-border">
                      <div
                        className="h-full bg-accent transition-[width] duration-150"
                        style={{ width: `${(verifyProgress.current / verifyProgress.total) * 100}%` }}
                      />
                    </div>
                  )}
                </div>
              )}
            </div>
          </section>
        </>
      )}

      {/* Platform selector dialog */}
      {showPlatformDialog && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
          <div className="w-[600px] bg-bg-card border border-border p-xl flex flex-col gap-xl">
            <div className="flex flex-col gap-sm">
              <h2 className="font-display text-xl font-bold text-text-primary">Select Platform</h2>
              <p className="text-body text-text-muted">Could not auto-detect platform for:</p>
              <p className="text-body text-accent font-semibold font-mono">{pendingDatHeaderName}</p>
            </div>

            <div className="flex items-center gap-md px-md py-sm bg-bg-elevated border border-border">
              <span className="text-text-dim">&#x1F50D;</span>
              <input
                type="text"
                className="flex-1 bg-transparent border-none outline-none text-body text-text-primary font-mono"
                placeholder="Search platforms..."
                value={platformSearch}
                onChange={(e) => setPlatformSearch(e.target.value)}
                autoFocus
              />
            </div>

            <div className="max-h-[240px] overflow-y-auto border border-border">
              {platforms
                .filter((p) =>
                  p.name.toLowerCase().includes(platformSearch.toLowerCase()) ||
                  p.slug.toLowerCase().includes(platformSearch.toLowerCase())
                )
                .map((p) => (
                  <button
                    key={p.id}
                    className="w-full flex items-center gap-md px-lg py-md text-left hover:bg-bg-elevated transition-colors border-b border-border last:border-b-0 bg-transparent cursor-pointer"
                    onClick={() => handlePlatformSelect(p.slug)}
                  >
                    <span className="text-body text-text-primary">{p.name}</span>
                    <span className="text-nav text-text-dim font-mono">{p.slug}</span>
                  </button>
                ))}
              {platforms.filter((p) =>
                p.name.toLowerCase().includes(platformSearch.toLowerCase()) ||
                p.slug.toLowerCase().includes(platformSearch.toLowerCase())
              ).length === 0 && (
                <div className="px-lg py-xl text-center text-body text-text-muted">
                  No platforms found
                </div>
              )}
            </div>

            <div className="flex justify-end gap-md">
              <button
                className="btn btn-secondary"
                onClick={() => {
                  setShowPlatformDialog(false);
                  setPendingDatPath(null);
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
