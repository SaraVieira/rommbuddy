import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { EmulatorDef, SavePathOverride } from "../../types";
import { toast } from "sonner";
import SaveDirectories from "./SaveDirectories";

export default function EmulatorsTab() {
  const [emulators, setEmulators] = useState<EmulatorDef[]>([]);
  const [emulatorPaths, setEmulatorPaths] = useState<Record<string, string>>(
    {},
  );
  const [savePaths, setSavePaths] = useState<Record<string, SavePathOverride>>(
    {},
  );

  const loadEmulators = useCallback(async () => {
    try {
      const emus: EmulatorDef[] = await invoke("get_emulators");
      setEmulators(emus);
      const paths: Record<string, string> = await invoke("get_emulator_paths");
      const detected: [string, string][] = await invoke("detect_emulators");
      const merged = { ...paths };
      for (const [id, path] of detected) {
        if (!merged[id]) {
          merged[id] = path;
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
      const paths =
        await invoke<Record<string, SavePathOverride>>("get_save_paths");
      setSavePaths(paths);
    } catch (e) {
      console.error("Failed to load save paths:", e);
    }
  }, []);

  useEffect(() => {
    loadEmulators();
    loadSavePaths();
  }, [loadEmulators, loadSavePaths]);

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
        toast.success("Emulator path saved");
      } catch (e) {
        toast.error(String(e));
      }
    }
  };

  return (
    <section>
      <p className="text-body text-text-muted mb-xl">
        Configure paths to standalone emulators. These can be used instead of
        RetroArch cores for specific platforms.
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
                emulatorPaths[emu.id]
                  ? "text-accent font-mono font-semibold uppercase"
                  : "text-text-muted font-mono font-semibold uppercase"
              }
            >
              {emulatorPaths[emu.id] ? "[found]" : "[not found]"}
            </span>
          </div>
        ))}
      </div>

      <SaveDirectories
        emulators={emulators}
        savePaths={savePaths}
        onSavePathsChange={setSavePaths}
      />
    </section>
  );
}
