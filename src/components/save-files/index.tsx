import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useSetAtom } from "jotai";
import { Upload } from "lucide-react";
import { toast } from "sonner";
import type { SaveFileInfo } from "../../types";
import { romSavesAtom } from "../../store/library";
import SaveFileRow from "./Row";
import SaveStateRow from "./StateRow";
import SectionHeading from "@/components/SectionHeading";

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

  const fetchSaves = useCallback(async () => {
    try {
      const result = await invoke<SaveFileInfo[]>("get_rom_saves", { romId });
      setSaves(result);
      setRomSaves((prev) => ({ ...prev, [romId]: result.length > 0 }));
    } catch (e) {
      console.error("Failed to load saves:", e);
      toast.error(String(e));
    }
  }, [romId, setRomSaves]);

  useEffect(() => {
    setLoading(true);
    fetchSaves().finally(() => setLoading(false));
  }, [fetchSaves]);

  const handleDelete = async (sf: SaveFileInfo) => {
    if (!confirm(`Delete ${sf.file_name}? This cannot be undone.`)) return;
    try {
      await invoke("delete_save_file", { filePath: sf.file_path });
      setSaves((prev) => prev.filter((s) => s.file_path !== sf.file_path));
      setRomSaves((prev) => ({ ...prev, [romId]: saves.length > 1 }));
    } catch (e) {
      console.error("Delete failed:", e);
      toast.error(String(e));
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
        toast.error(String(e));
      }
    }
  };

  const handleImport = async () => {
    const selected = await open({ multiple: false, title: "Import save file" });
    if (selected) {
      const firstSave = saves[0];
      const destDir = firstSave
        ? firstSave.file_path.substring(0, firstSave.file_path.lastIndexOf("/"))
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
        toast.error(String(e));
      }
    }
  };

  if (loading) {
    return (
      <div className="flex flex-col gap-lg">
        <SectionHeading size="sm">Saves</SectionHeading>
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
          <SectionHeading size="sm">Saves</SectionHeading>
          <button
            onClick={handleImport}
            className="flex items-center gap-xs font-mono text-[10px] font-semibold text-text-secondary hover:text-text-primary uppercase"
          >
            <Upload size={10} />
            Import
          </button>
        </div>
        <span className="font-mono text-[10px] text-text-muted uppercase">
          No saves found
        </span>
      </div>
    );
  }

  const saveFiles = saves.filter((s) => s.save_type === "save_file");
  const saveStates = saves.filter((s) => s.save_type === "save_state");

  return (
    <div className="flex flex-col gap-lg">
      <div className="flex items-center justify-between">
        <SectionHeading size="sm">Saves</SectionHeading>
        <div className="flex items-center gap-lg">
          <span className="font-mono text-[10px] text-text-muted uppercase">
            {saves.length} {saves.length === 1 ? "file" : "files"}
          </span>
          <button
            onClick={handleImport}
            className="flex items-center gap-xs font-mono text-[10px] font-semibold text-text-secondary hover:text-text-primary uppercase"
          >
            <Upload size={10} />
            Import
          </button>
        </div>
      </div>

      {saveFiles.length > 0 && (
        <div className="flex flex-col gap-md">
          <span className="font-mono text-[10px] font-semibold text-text-muted uppercase">
            Save Files
          </span>
          <div className="flex flex-col gap-xs">
            {saveFiles.map((sf) => (
              <SaveFileRow
                key={sf.file_path}
                save={sf}
                onExport={handleExport}
                onDelete={handleDelete}
              />
            ))}
          </div>
        </div>
      )}

      {saveStates.length > 0 && (
        <div className="flex flex-col gap-md">
          <span className="font-mono text-[10px] font-semibold text-text-muted uppercase">
            Save States
          </span>
          <div className="flex flex-col gap-xs">
            {saveStates.map((ss) => (
              <SaveStateRow
                key={ss.file_path}
                save={ss}
                onExport={handleExport}
                onDelete={handleDelete}
                onLaunch={onLaunchSaveState}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
