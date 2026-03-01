interface Props {
  current: number;
  total: number;
  label?: string;
  formatBytes?: boolean;
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** i;
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export default function ProgressBar({ current, total, label, formatBytes }: Props) {
  const pct = total > 0 ? Math.round((current / total) * 100) : 0;
  const fmt = formatBytes ? formatSize : (n: number) => String(n);

  return (
    <div className="flex flex-col gap-sm">
      {label && (
        <div className="text-body text-text-secondary overflow-hidden text-ellipsis whitespace-nowrap">
          {label}
        </div>
      )}
      <div className="h-2 bg-bg-elevated rounded-none overflow-hidden">
        <div
          className="h-full bg-accent rounded-none transition-[width] duration-200 ease-out shadow-accent-glow"
          style={{ width: `${pct}%` }}
        />
      </div>
      <div className="text-nav text-text-muted">
        {fmt(current)} / {fmt(total)}{total > 0 ? ` (${pct}%)` : ""}
      </div>
    </div>
  );
}
