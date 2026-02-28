import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { useSetAtom } from "jotai";
import type { PlatformWithCount } from "../types";
import { getPlatformIcon } from "../utils/platformIcons";
import {
  selectedPlatformAtom,
  offsetAtom,
  searchInputAtom,
  searchAtom,
} from "../store/library";

export default function Platforms() {
  const navigate = useNavigate();
  const setSelectedPlatform = useSetAtom(selectedPlatformAtom);
  const setOffset = useSetAtom(offsetAtom);
  const setSearchInput = useSetAtom(searchInputAtom);
  const setSearch = useSetAtom(searchAtom);

  const [platforms, setPlatforms] = useState<PlatformWithCount[]>([]);
  const [filter, setFilter] = useState("");
  const [loading, setLoading] = useState(false);

  const loadPlatforms = useCallback(async () => {
    setLoading(true);
    try {
      const result: PlatformWithCount[] = await invoke(
        "get_platforms_with_counts",
      );
      setPlatforms(result);
    } catch (e) {
      console.error("Failed to load platforms:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadPlatforms();
  }, [loadPlatforms]);

  const filtered = platforms.filter((p) =>
    p.name.toLowerCase().includes(filter.toLowerCase()),
  );

  const handlePlatformClick = (platformId: number) => {
    setSelectedPlatform(platformId);
    setOffset(0);
    setSearchInput("");
    setSearch("");
    navigate("/");
  };
  console.log("platform", platforms);

  return (
    <div className="page">
      <div className="flex flex-wrap justify-between items-end gap-xl mb-3xl">
        <div className="flex flex-col gap-sm">
          <h1 className="font-display text-page-title font-bold text-text-primary uppercase">
            Platforms
          </h1>
          <span className="text-nav text-text-muted">
            {platforms.length} platforms configured
          </span>
        </div>
        <input
          type="text"
          className="min-w-[180px] max-w-[280px] flex-1 px-xl py-[10px] rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-nav placeholder:text-text-dim focus:border-accent outline-none transition-[border-color] duration-150"
          placeholder="Filter platforms..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
      </div>

      {loading ? (
        <div className="text-center p-[40px] text-text-muted">Loading...</div>
      ) : filtered.length === 0 ? (
        <div className="text-center p-[40px] text-text-muted">
          No platforms found.
        </div>
      ) : (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(176px,1fr))] gap-xl">
          {filtered.map((platform) => {
            const iconSrc = getPlatformIcon(platform.slug);
            return (
              <button
                key={platform.id}
                className="flex flex-col items-center gap-lg p-3xl bg-bg-card border border-border rounded-none hover:border-border-accent-tint hover:bg-bg-elevated font-mono text-text-primary cursor-pointer transition-all"
                onClick={() => handlePlatformClick(platform.id)}
              >
                <div className="w-[48px] h-[48px] flex items-center justify-center bg-bg-elevated rounded-none">
                  {iconSrc ? (
                    <img
                      src={iconSrc}
                      alt={platform.name}
                      className="w-[40px] h-[40px] object-contain [image-rendering:pixelated]"
                    />
                  ) : (
                    <span className="text-subtitle font-bold text-accent tracking-[1px] uppercase">
                      {platform.slug.substring(0, 2)}
                    </span>
                  )}
                </div>
                <span className="text-body font-semibold tracking-wide uppercase">
                  {platform.name}
                </span>
                <span className="text-badge text-text-muted">
                  {platform.rom_count} ROMs
                </span>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
