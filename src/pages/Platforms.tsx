import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAtomValue, useSetAtom } from "jotai";
import { getPlatformIcon } from "../utils/platformIcons";
import {
  selectedPlatformAtom,
  searchInputAtom,
  searchAtom,
} from "../store/library";
import { platformsAtom } from "../store/platforms";
import SearchInput from "../components/SearchInput";

export default function Platforms() {
  const navigate = useNavigate();
  const setSelectedPlatform = useSetAtom(selectedPlatformAtom);
  const setSearchInput = useSetAtom(searchInputAtom);
  const setSearch = useSetAtom(searchAtom);

  const platforms = useAtomValue(platformsAtom);
  const [filter, setFilter] = useState("");

  const filtered = platforms.filter((p) =>
    p.name.toLowerCase().includes(filter.toLowerCase()),
  );

  const handlePlatformClick = (platformId: number) => {
    setSelectedPlatform(platformId);
    setSearchInput("");
    setSearch("");
    navigate("/", { replace: true });
  };

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
        <SearchInput
          className="min-w-[180px] max-w-[280px] flex-1 text-nav"
          placeholder="Filter platforms..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
      </div>

      {filtered.length === 0 ? (
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
