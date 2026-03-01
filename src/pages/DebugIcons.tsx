import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getPlatformIcon, ICON_MAP } from "../utils/platformIcons";

type RegistryPlatform = [string, string]; // [slug, display_name]

export default function DebugIcons() {
  const [platforms, setPlatforms] = useState<RegistryPlatform[]>([]);
  const [filter, setFilter] = useState<"all" | "missing" | "has-icon">("all");
  const [search, setSearch] = useState("");

  useEffect(() => {
    invoke<RegistryPlatform[]>("get_all_registry_platforms").then(setPlatforms);
  }, []);

  const { items, missing, hasIcon } = useMemo(() => {
    const sorted = [...platforms].sort((a, b) => a[1].localeCompare(b[1]));
    const items: { slug: string; displayName: string; iconPath: string | null; hasIcon: boolean }[] = [];
    const missing: typeof items = [];
    const hasIcon: typeof items = [];
    for (const [slug, displayName] of sorted) {
      const item = {
        slug,
        displayName,
        iconPath: getPlatformIcon(slug),
        hasIcon: slug in ICON_MAP,
      };
      items.push(item);
      (item.hasIcon ? hasIcon : missing).push(item);
    }
    return { items, missing, hasIcon };
  }, [platforms]);

  const byFilter =
    filter === "missing" ? missing : filter === "has-icon" ? hasIcon : items;

  const filtered = search
    ? byFilter.filter(
        (i) =>
          i.displayName.toLowerCase().includes(search.toLowerCase()) ||
          i.slug.toLowerCase().includes(search.toLowerCase()),
      )
    : byFilter;

  return (
    <div className="page">
      <div className="flex flex-wrap justify-between items-end gap-xl mb-3xl">
        <div className="flex flex-col gap-sm">
          <h1 className="font-display text-page-title font-bold text-text-primary uppercase">
            Debug: Platform Icons
          </h1>
          <span className="text-nav text-text-muted">
            {platforms.length} registered &middot;{" "}
            <span className="text-accent">{hasIcon.length} have icons</span>
            {" "}&middot;{" "}
            <span className="text-red-500">{missing.length} missing</span>
          </span>
        </div>
        <div className="flex items-end gap-lg">
          <input
            type="text"
            className="min-w-[180px] max-w-[280px] flex-1 px-xl py-[10px] rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-nav placeholder:text-text-dim focus:border-accent outline-none transition-[border-color] duration-150"
            placeholder="Search platforms..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <div className="flex gap-md">
          {(
            [
              ["all", items.length],
              ["missing", missing.length],
              ["has-icon", hasIcon.length],
            ] as const
          ).map(([f, count]) => (
            <button
              key={f}
              onClick={() => setFilter(f as typeof filter)}
              className={`px-xl py-sm font-mono text-badge uppercase border ${
                filter === f
                  ? "border-accent text-accent bg-accent/10"
                  : "border-border text-text-muted bg-bg-elevated hover:border-border-light"
              }`}
            >
              {f} ({count})
            </button>
          ))}
        </div>
      </div>

      <div className="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-lg">
        {filtered.map((item) => (
          <div
            key={item.slug}
            className={`flex flex-col items-center gap-md p-xl border ${
              item.hasIcon
                ? "border-border bg-bg-card"
                : "border-red-500/40 bg-red-500/5"
            }`}
          >
            <div className="w-[48px] h-[48px] flex items-center justify-center bg-bg-elevated">
              {item.iconPath ? (
                <img
                  src={item.iconPath}
                  alt={item.displayName}
                  className="w-[40px] h-[40px] object-contain [image-rendering:pixelated]"
                />
              ) : (
                <span className="text-[10px] font-mono text-red-500 uppercase">
                  N/A
                </span>
              )}
            </div>
            <span className="text-badge font-mono text-text-primary text-center uppercase leading-tight">
              {item.displayName}
            </span>
            <span className="text-[10px] font-mono text-text-dim">
              {item.slug}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
