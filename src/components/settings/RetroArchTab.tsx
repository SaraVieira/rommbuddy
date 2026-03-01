import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  PlatformWithCount,
  CoreInfo,
  CoreMapping,
  EmulatorDef,
} from "../../types";
import { toast } from "sonner";
import CoreMappings from "./CoreMappings";
import InstallCores from "./InstallCores";
import { DEFAULT_CORES } from "../../utils/defaultCores";

export default function RetroArchTab() {
  const [retroarchPath, setRetroarchPath] = useState("");
  const [pathValid, setPathValid] = useState(false);
  const [cores, setCores] = useState<CoreInfo[]>([]);
  const [platforms, setPlatforms] = useState<PlatformWithCount[]>([]);
  const [mappings, setMappings] = useState<CoreMapping[]>([]);
  const [emulators, setEmulators] = useState<EmulatorDef[]>([]);
  const [emulatorPaths, setEmulatorPaths] = useState<Record<string, string>>(
    {},
  );

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

  const loadEmulators = useCallback(async () => {
    try {
      const emus: EmulatorDef[] = await invoke("get_emulators");
      setEmulators(emus);
      const paths: Record<string, string> = await invoke("get_emulator_paths");
      setEmulatorPaths(paths);
    } catch (e) {
      console.error("Failed to load emulators:", e);
    }
  }, []);

  useEffect(() => {
    loadSettings();
    loadMappings();
    loadEmulators();
  }, [loadSettings, loadMappings, loadEmulators]);

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
      toast.success("RetroArch path saved");
      const detected: CoreInfo[] = await invoke("detect_cores", {
        retroarchPath: path,
      });
      setCores(detected);
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
      toast.error(String(e));
    }
  };

  return (
    <>
      <section>
        <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">
          // RetroArch Path
        </h2>
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
              <span
                className={
                  pathValid
                    ? "text-accent font-mono font-semibold uppercase"
                    : "text-error font-mono font-semibold uppercase"
                }
              >
                {pathValid ? "[found]" : "[not found]"}
              </span>
            )}
          </div>
        </div>
      </section>

      <CoreMappings
        platforms={platforms}
        cores={cores}
        mappings={mappings}
        emulators={emulators}
        emulatorPaths={emulatorPaths}
        pathValid={pathValid}
        onRefresh={loadMappings}
      />

      {pathValid && (
        <InstallCores retroarchPath={retroarchPath} onCoresChanged={setCores} />
      )}
    </>
  );
}
