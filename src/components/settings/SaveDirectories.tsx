import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { EmulatorDef, SavePathOverride } from "../../types";
import { useAppToast } from "../../App";

interface SaveDirectoriesProps {
  emulators: EmulatorDef[];
  savePaths: Record<string, SavePathOverride>;
  onSavePathsChange: (paths: Record<string, SavePathOverride>) => void;
}

export default function SaveDirectories({
  emulators,
  savePaths,
  onSavePathsChange,
}: SaveDirectoriesProps) {
  const toast = useAppToast();

  const handleBrowse = async (
    emulatorId: string,
    dirType: "save_dir" | "state_dir",
  ) => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: `Select ${dirType === "save_dir" ? "save files" : "save states"} directory for ${emulatorId}`,
    });
    if (selected) {
      const existing = savePaths[emulatorId] || {
        save_dir: null,
        state_dir: null,
      };
      const updated = { ...existing, [dirType]: selected as string };
      try {
        await invoke("set_save_path", {
          emulatorId,
          saveDir: updated.save_dir,
          stateDir: updated.state_dir,
        });
        onSavePathsChange({ ...savePaths, [emulatorId]: updated });
        toast("Save directory saved", "success");
      } catch (e) {
        toast(String(e), "error");
      }
    }
  };

  const handleReset = async (emulatorId: string) => {
    try {
      await invoke("set_save_path", {
        emulatorId,
        saveDir: null,
        stateDir: null,
      });
      const next = { ...savePaths };
      delete next[emulatorId];
      onSavePathsChange(next);
      toast("Reset to default save directories", "success");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const emuIds = ["retroarch", ...emulators.map((e) => e.id)];

  return (
    <div className="mt-3xl">
      <h2 className="font-mono text-[13px] font-semibold text-accent uppercase tracking-wide mb-lg">
        // SAVE DIRECTORIES
      </h2>
      <div className="bg-bg-card border border-border p-3xl flex flex-col gap-xl">
        <p className="font-mono text-[12px] text-text-muted leading-[1.6]">
          Override the default save file and save state directories for each
          emulator. Leave blank to use each emulator's default paths.
        </p>

        {emuIds.map((emuId) => {
          const override = savePaths[emuId];
          const hasCustom =
            override && (override.save_dir || override.state_dir);
          return (
            <div
              key={emuId}
              className="flex flex-col gap-md border border-border p-xl"
            >
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
                      onClick={() => handleReset(emuId)}
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

              {(["save_dir", "state_dir"] as const).map((dirType) => (
                <div key={dirType} className="flex flex-col gap-sm">
                  <span className="font-mono text-[10px] font-semibold text-text-muted uppercase">
                    {dirType === "save_dir" ? "SAVES" : "STATES"}
                  </span>
                  <div className="flex gap-md">
                    <input
                      type="text"
                      className="flex-1 bg-bg-elevated border border-border font-mono text-[11px] text-text-primary px-lg py-sm"
                      value={override?.[dirType] || ""}
                      readOnly
                      placeholder={`Default ${dirType === "save_dir" ? "save" : "save states"} directory`}
                    />
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={() => handleBrowse(emuId, dirType)}
                    >
                      BROWSE
                    </button>
                  </div>
                </div>
              ))}
            </div>
          );
        })}
      </div>
    </div>
  );
}
