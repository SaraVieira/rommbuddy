import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useSetAtom } from "jotai";
import { Save, Bookmark, Trash2, Download, Upload, Play } from "lucide-react";
import type { SaveFileInfo } from "../types";
import { romSavesAtom } from "../store/library";

function ScreenshotImg({ path }: { path: string }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    invoke<string>("read_file_base64", { filePath: path }).then(setSrc).catch(() => {});
  }, [path]);
  if (!src) {
    return (
      <div className="w-16 h-12 bg-bg-elevated border border-border flex items-center justify-center shrink-0">
        <span className="font-mono text-[9px] text-text-muted">IMG</span>
      </div>
    );
  }
  return (
    <img
      src={src}
      className="w-16 h-12 object-cover border border-border shrink-0"
      alt="Save state screenshot"
    />
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  } catch {
    return iso;
  }
}

export function SaveFiles({
  romId,
  onLaunchSaveState,
}: {
  romId: number;
  onLaunchSaveState?: (slot: number | null, filePath: string) => void;
}) {
  const [saves, setSaves] = useState<SaveFileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const setRomSaves = useSetAtom(romSavesAtom);

  const fetchSaves = async () => {
    try {
      const result = await invoke<SaveFileInfo[]>("get_rom_saves", { romId });
      setSaves(result);
      setRomSaves((prev) => ({ ...prev, [romId]: result.length > 0 }));
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    (async () => {
      await fetchSaves();
      if (!cancelled) setLoading(false);
    })();
    return () => {
      cancelled = true;
    };
  }, [romId]);

  const handleDelete = async (sf: SaveFileInfo) => {
    if (!confirm(`Delete ${sf.file_name}? This cannot be undone.`)) return;
    try {
      await invoke("delete_save_file", { filePath: sf.file_path });
      setSaves((prev) => prev.filter((s) => s.file_path !== sf.file_path));
      setRomSaves((prev) => ({
        ...prev,
        [romId]: saves.length > 1,
      }));
    } catch (e) {
      console.error("Delete failed:", e);
    }
  };

  const handleExport = async (sf: SaveFileInfo) => {
    const dest = await save({
      defaultPath: sf.file_name,
      title: "Export save file",
    });
    if (dest) {
      try {
        await invoke("export_save_file", {
          sourcePath: sf.file_path,
          destPath: dest,
        });
      } catch (e) {
        console.error("Export failed:", e);
      }
    }
  };

  const handleImport = async () => {
    const selected = await open({
      multiple: false,
      title: "Import save file",
    });
    if (selected) {
      // Get the directory of the first existing save, or fall back to re-fetch
      const firstSave = saves[0];
      const destDir = firstSave
        ? firstSave.file_path.substring(
            0,
            firstSave.file_path.lastIndexOf("/")
          )
        : null;

      if (!destDir) return;

      const fileName = (selected as string).split("/").pop() || "save";
      try {
        await invoke("import_save_file", {
          sourcePath: selected as string,
          destDir,
          fileName,
        });
        await fetchSaves();
      } catch (e) {
        console.error("Import failed:", e);
      }
    }
  };

  if (loading) {
    return (
      <div className="flex flex-col gap-lg">
        <span className="font-mono text-[11px] font-semibold text-accent tracking-[0.5px]">
          // SAVES
        </span>
        <span className="font-mono text-[10px] text-text-muted">
          Scanning...
        </span>
      </div>
    );
  }

  if (saves.length === 0) {
    return (
      <div className="flex flex-col gap-lg">
        <div className="flex items-center justify-between">
          <span className="font-mono text-[11px] font-semibold text-accent tracking-[0.5px]">
            // SAVES
          </span>
          <button
            onClick={handleImport}
            className="flex items-center gap-xs font-mono text-[10px] font-semibold text-text-secondary hover:text-text-primary"
          >
            <Upload size={10} />
            IMPORT
          </button>
        </div>
        <span className="font-mono text-[10px] text-text-muted">
          NO SAVES FOUND
        </span>
      </div>
    );
  }

  const saveFiles = saves.filter((s) => s.save_type === "save_file");
  const saveStates = saves.filter((s) => s.save_type === "save_state");

  return (
    <div className="flex flex-col gap-lg">
      <div className="flex items-center justify-between">
        <span className="font-mono text-[11px] font-semibold text-accent tracking-[0.5px]">
          // SAVES
        </span>
        <div className="flex items-center gap-lg">
          <span className="font-mono text-[10px] text-text-muted">
            {saves.length} {saves.length === 1 ? "FILE" : "FILES"}
          </span>
          <button
            onClick={handleImport}
            className="flex items-center gap-xs font-mono text-[10px] font-semibold text-text-secondary hover:text-text-primary"
          >
            <Upload size={10} />
            IMPORT
          </button>
        </div>
      </div>

      {saveFiles.length > 0 && (
        <div className="flex flex-col gap-md">
          <span className="font-mono text-[10px] font-semibold text-text-muted">
            SAVE FILES
          </span>
          <div className="flex flex-col gap-xs">
            {saveFiles.map((sf) => (
              <div
                key={sf.file_path}
                className="flex items-center gap-lg bg-bg-card border border-border px-xl py-lg group"
              >
                <Save size={14} className="text-text-secondary shrink-0" />
                <span className="font-mono text-[11px] font-medium text-text-primary truncate">
                  {sf.file_name}
                </span>
                <span className="font-mono text-[10px] text-text-muted ml-auto shrink-0">
                  {formatSize(sf.size_bytes)} &middot;{" "}
                  {formatDate(sf.modified_at)}
                </span>
                <button
                  onClick={() => handleExport(sf)}
                  className="text-text-muted hover:text-text-primary opacity-0 group-hover:opacity-100 shrink-0"
                  title="Export"
                >
                  <Download size={12} />
                </button>
                <button
                  onClick={() => handleDelete(sf)}
                  className="text-text-muted hover:text-red-500 opacity-0 group-hover:opacity-100 shrink-0"
                  title="Delete"
                >
                  <Trash2 size={12} />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {saveStates.length > 0 && (
        <div className="flex flex-col gap-md">
          <span className="font-mono text-[10px] font-semibold text-text-muted">
            SAVE STATES
          </span>
          <div className="flex flex-col gap-xs">
            {saveStates.map((ss) => (
              <div
                key={ss.file_path}
                className="flex items-center gap-lg bg-bg-card border border-border px-xl py-lg group"
              >
                {ss.screenshot_path ? (
                  <ScreenshotImg path={ss.screenshot_path} />
                ) : (
                  <div className="w-16 h-12 bg-bg-elevated border border-border flex items-center justify-center shrink-0">
                    <span className="font-mono text-[9px] text-text-muted">
                      IMG
                    </span>
                  </div>
                )}
                <Bookmark
                  size={14}
                  className="text-text-secondary shrink-0"
                />
                <span className="font-mono text-[11px] font-medium text-text-primary truncate">
                  {ss.file_name}
                </span>
                {ss.slot != null && (
                  <span className="font-mono text-[9px] font-semibold text-accent bg-accent/15 border border-accent/30 px-md py-xs shrink-0">
                    SLOT {ss.slot}
                  </span>
                )}
                <span className="font-mono text-[10px] text-text-muted ml-auto shrink-0">
                  {formatSize(ss.size_bytes)} &middot;{" "}
                  {formatDate(ss.modified_at)}
                </span>
                {onLaunchSaveState && (
                  <button
                    onClick={() => onLaunchSaveState(ss.slot, ss.file_path)}
                    className="text-accent hover:text-accent/80 opacity-0 group-hover:opacity-100 shrink-0"
                    title={ss.slot != null ? `Launch from slot ${ss.slot}` : "Launch from save state"}
                  >
                    <Play size={12} />
                  </button>
                )}
                <button
                  onClick={() => handleExport(ss)}
                  className="text-text-muted hover:text-text-primary opacity-0 group-hover:opacity-100 shrink-0"
                  title="Export"
                >
                  <Download size={12} />
                </button>
                <button
                  onClick={() => handleDelete(ss)}
                  className="text-text-muted hover:text-red-500 opacity-0 group-hover:opacity-100 shrink-0"
                  title="Delete"
                >
                  <Trash2 size={12} />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
