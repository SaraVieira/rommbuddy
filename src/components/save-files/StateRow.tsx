import { Bookmark, Download, Trash2, Play } from "lucide-react";
import type { SaveFileInfo } from "../../types";
import { formatSize, formatDate } from "../../utils/format";
import ScreenshotImg from "./ScreenshotImg";

interface SaveStateRowProps {
  save: SaveFileInfo;
  onExport: (save: SaveFileInfo) => void;
  onDelete: (save: SaveFileInfo) => void;
  onLaunch?: (slot: number | null, filePath: string) => void;
}

export default function SaveStateRow({
  save,
  onExport,
  onDelete,
  onLaunch,
}: SaveStateRowProps) {
  return (
    <div className="flex items-center gap-lg bg-bg-card border border-border px-xl py-lg group">
      {save.screenshot_path ? (
        <ScreenshotImg path={save.screenshot_path} />
      ) : (
        <div className="w-16 h-12 bg-bg-elevated border border-border flex items-center justify-center shrink-0">
          <span className="font-mono text-[9px] text-text-muted">IMG</span>
        </div>
      )}
      <Bookmark size={14} className="text-text-secondary shrink-0" />
      <span className="font-mono text-[11px] font-medium text-text-primary truncate">
        {save.file_name}
      </span>
      {save.slot != null && (
        <span className="font-mono text-[9px] font-semibold text-accent bg-accent/15 border border-accent/30 px-md py-xs shrink-0 uppercase">
          Slot {save.slot}
        </span>
      )}
      <span className="font-mono text-[10px] text-text-muted ml-auto shrink-0">
        {formatSize(save.size_bytes)} &middot; {formatDate(save.modified_at)}
      </span>
      {onLaunch && (
        <button
          onClick={() => onLaunch(save.slot, save.file_path)}
          className="text-accent hover:text-accent/80 opacity-0 group-hover:opacity-100 shrink-0"
          title={
            save.slot != null
              ? `Launch from slot ${save.slot}`
              : "Launch from save state"
          }
        >
          <Play size={12} />
        </button>
      )}
      <button
        onClick={() => onExport(save)}
        className="text-text-muted hover:text-text-primary opacity-0 group-hover:opacity-100 shrink-0"
        title="Export"
      >
        <Download size={12} />
      </button>
      <button
        onClick={() => onDelete(save)}
        className="text-text-muted hover:text-red-500 opacity-0 group-hover:opacity-100 shrink-0"
        title="Delete"
      >
        <Trash2 size={12} />
      </button>
    </div>
  );
}
