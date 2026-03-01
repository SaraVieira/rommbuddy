import type { ScanProgress } from "../types";

interface Props {
  label: string;
  progress: ScanProgress;
  color?: string;
  onCancel: () => void;
}

export default function OperationProgressBanner({
  label,
  progress,
  color = "text-accent",
  onCancel,
}: Props) {
  const pct =
    progress.total > 0
      ? Math.round((progress.current / progress.total) * 100)
      : 0;
  const barColor = color === "text-accent" ? "bg-accent" : "bg-yellow-400";
  const glowClass = color === "text-accent" ? "shadow-accent-glow" : "";

  return (
    <div className="sticky top-0 z-50 flex items-center gap-xl px-6xl py-md bg-bg-sidebar border-b border-border">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-lg mb-xs">
          <span
            className={`text-badge font-mono uppercase tracking-[1px] shrink-0 ${color}`}
          >
            {label}
          </span>
          <span className="text-badge font-mono text-text-muted truncate">
            {progress.current_item}
          </span>
          <span className="text-badge font-mono text-text-secondary shrink-0">
            {progress.current} / {progress.total}
            {progress.total > 0 && ` (${pct}%)`}
          </span>
        </div>
        <div className="h-1 bg-bg-elevated overflow-hidden">
          <div
            className={`h-full transition-[width] duration-200 ease-out ${barColor} ${glowClass}`}
            style={{ width: `${pct}%` }}
          />
        </div>
      </div>
      <button className="btn btn-secondary btn-sm shrink-0" onClick={onCancel}>
        Cancel
      </button>
    </div>
  );
}
