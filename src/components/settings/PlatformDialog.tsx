import { useState } from "react";
import type { PlatformWithCount } from "../../types";

interface PlatformDialogProps {
  platforms: PlatformWithCount[];
  headerName: string;
  onSelect: (slug: string) => void;
  onCancel: () => void;
}

export default function PlatformDialog({
  platforms,
  headerName,
  onSelect,
  onCancel,
}: PlatformDialogProps) {
  const [search, setSearch] = useState("");

  const filtered = platforms.filter(
    (p) =>
      p.name.toLowerCase().includes(search.toLowerCase()) ||
      p.slug.toLowerCase().includes(search.toLowerCase()),
  );

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="w-[600px] bg-bg-card border border-border p-xl flex flex-col gap-xl">
        <div className="flex flex-col gap-sm">
          <h2 className="font-display text-xl font-bold text-text-primary">
            Select Platform
          </h2>
          <p className="text-body text-text-muted">
            Could not auto-detect platform for:
          </p>
          <p className="text-body text-accent font-semibold font-mono">
            {headerName}
          </p>
        </div>

        <div className="flex items-center gap-md px-md py-sm bg-bg-elevated border border-border">
          <span className="text-text-dim">&#x1F50D;</span>
          <input
            type="text"
            className="flex-1 bg-transparent border-none outline-none text-body text-text-primary font-mono"
            placeholder="Search platforms..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            autoFocus
          />
        </div>

        <div className="max-h-[240px] overflow-y-auto border border-border">
          {filtered.map((p) => (
            <button
              key={p.id}
              className="w-full flex items-center gap-md px-lg py-md text-left hover:bg-bg-elevated transition-colors border-b border-border last:border-b-0 bg-transparent cursor-pointer"
              onClick={() => onSelect(p.slug)}
            >
              <span className="text-body text-text-primary">{p.name}</span>
              <span className="text-nav text-text-dim font-mono">{p.slug}</span>
            </button>
          ))}
          {filtered.length === 0 && (
            <div className="px-lg py-xl text-center text-body text-text-muted">
              No platforms found
            </div>
          )}
        </div>

        <div className="flex justify-end gap-md">
          <button className="btn btn-secondary" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
