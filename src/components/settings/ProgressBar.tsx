import type { ScanProgress } from "../../types";

export default function ProgressBar({ progress }: { progress: ScanProgress }) {
  return (
    <div className="mt-md text-body text-text-muted">
      <div className="flex items-center justify-between">
        <span className="truncate mr-md">{progress.current_item}</span>
        {progress.total > 0 && (
          <span className="shrink-0">
            {progress.current} / {progress.total}
          </span>
        )}
      </div>
      {progress.total > 1 && (
        <div className="mt-xs h-0.75 bg-border">
          <div
            className="h-full bg-accent transition-[width] duration-150"
            style={{
              width: `${(progress.current / progress.total) * 100}%`,
            }}
          />
        </div>
      )}
    </div>
  );
}
