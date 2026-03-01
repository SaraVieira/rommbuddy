import { formatSize } from "../utils/format";

interface Props {
  current: number;
  total: number;
  label?: string;
  currentItem?: string;
  formatBytes?: boolean;
  color?: string;
}

export default function ProgressBar({
  current,
  total,
  label,
  currentItem,
  formatBytes,
  color = "bg-accent",
}: Props) {
  const pct = total > 0 ? Math.round((current / total) * 100) : 0;
  const fmt = formatBytes ? formatSize : (n: number) => String(n);

  return (
    <div className="flex flex-col gap-sm">
      {label && (
        <div className="text-body text-text-secondary overflow-hidden text-ellipsis whitespace-nowrap">
          {label}
        </div>
      )}
      {currentItem && (
        <div className="flex items-center justify-between text-body text-text-muted">
          <span className="truncate mr-md">{currentItem}</span>
          {total > 0 && (
            <span className="shrink-0">
              {current} / {total}
            </span>
          )}
        </div>
      )}
      {total > 0 && (
        <div className="h-2 bg-bg-elevated rounded-none overflow-hidden">
          <div
            className={`h-full rounded-none transition-[width] duration-200 ease-out ${color}`}
            style={{ width: `${pct}%` }}
          />
        </div>
      )}
      {!currentItem && (
        <div className="text-nav text-text-muted">
          {fmt(current)} / {fmt(total)}{total > 0 ? ` (${pct}%)` : ""}
        </div>
      )}
    </div>
  );
}
