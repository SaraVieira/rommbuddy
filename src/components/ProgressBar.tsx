interface Props {
  current: number;
  total: number;
  label?: string;
}

export default function ProgressBar({ current, total, label }: Props) {
  const pct = total > 0 ? Math.round((current / total) * 100) : 0;

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
        {current} / {total} ({pct}%)
      </div>
    </div>
  );
}
