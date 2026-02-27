import { Save, Download, Trash2 } from "lucide-react";
import type { SaveFileInfo } from "../../types";
import { formatSize, formatDate } from "../../utils/format";

interface SaveFileRowProps {
  save: SaveFileInfo;
  onExport: (save: SaveFileInfo) => void;
  onDelete: (save: SaveFileInfo) => void;
}

export default function SaveFileRow({
  save,
  onExport,
  onDelete,
}: SaveFileRowProps) {
  return (
    <div className="flex items-center gap-lg bg-bg-card border border-border px-xl py-lg group">
      <Save size={14} className="text-text-secondary shrink-0" />
      <span className="font-mono text-[11px] font-medium text-text-primary truncate">
        {save.file_name}
      </span>
      <span className="font-mono text-[10px] text-text-muted ml-auto shrink-0">
        {formatSize(save.size_bytes)} &middot; {formatDate(save.modified_at)}
      </span>
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
