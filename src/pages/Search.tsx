import { useState, useEffect, useCallback, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { useAtomValue } from "jotai";
import type { RomWithMeta } from "../types";
import RomGrid from "../components/rom/Grid";
import RomList from "../components/rom/List";
import PlatformFilter from "../components/PlatformFilter";
import SearchInput from "../components/SearchInput";
import ViewToggle from "../components/ViewToggle";
import { platformsAtom } from "../store/platforms";
import { usePaginatedRoms } from "../hooks/usePaginatedRoms";

export default function Search() {
  const navigate = useNavigate();
  const inputRef = useRef<HTMLInputElement>(null);

  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const platforms = useAtomValue(platformsAtom);
  const [selectedPlatform, setSelectedPlatform] = useState<number | null>(null);
  const [view, setView] = useState<"grid" | "list">("grid");

  const searchEnabled = !!(debouncedQuery.trim() || selectedPlatform !== null);

  const {
    roms, total, loading, loadingMore, hasMore, loadMore, setRoms,
  } = usePaginatedRoms({
    platformId: selectedPlatform,
    search: debouncedQuery || null,
    enabled: searchEnabled,
  });

  // Auto-focus on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Debounce
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(query);
    }, 300);
    return () => clearTimeout(timer);
  }, [query]);

  const handleSelectRom = (rom: RomWithMeta) => {
    navigate(`/rom/${rom.id}`, { state: { rom } });
  };

  const handleToggleFavorite = useCallback(
    (romId: number, favorite: boolean) => {
      setRoms((prev) =>
        prev.map((r) => (r.id === romId ? { ...r, favorite } : r)),
      );
    },
    [],
  );

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex flex-col gap-xs mb-xl shrink-0">
        <h1 className="font-display text-page-title font-bold text-text-primary uppercase">
          Search
        </h1>
        <span className="text-nav text-text-muted">
          Search across all platforms
        </span>
      </div>
      <div className="flex items-center gap-md mb-3xl shrink-0">
        <SearchInput
          ref={inputRef}
          className="min-w-[160px] flex-1 max-w-[400px]"
          placeholder="Search ROMs..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <PlatformFilter
          platforms={platforms}
          selected={selectedPlatform}
          onSelect={(id) => {
            setSelectedPlatform(id);
          }}
        />
        <ViewToggle view={view} onChange={setView} />
      </div>

      {searchEnabled && !loading && roms.length > 0 && (
        <div className="text-nav text-text-muted mb-lg shrink-0">
          Found {total} result{total !== 1 ? "s" : ""}
          {debouncedQuery ? <> for &ldquo;{debouncedQuery}&rdquo;</> : null}
        </div>
      )}

      <div className="flex-1 min-h-0">
        {!searchEnabled ? (
          <div className="text-center py-[60px] px-[20px] text-text-dim text-[15px]">
            Search by name or select a platform.
          </div>
        ) : loading ? (
          <div className="text-center p-[40px] text-text-muted">
            Searching...
          </div>
        ) : roms.length === 0 ? (
          <div className="text-center p-[40px] text-text-muted">
            No results
            {debouncedQuery ? <> for &ldquo;{debouncedQuery}&rdquo;</> : null}
          </div>
        ) : view === "grid" ? (
          <RomGrid
            roms={roms}
            onSelect={handleSelectRom}
            onToggleFavorite={handleToggleFavorite}
            onLoadMore={loadMore}
            hasMore={hasMore}
            loadingMore={loadingMore}
          />
        ) : (
          <RomList
            roms={roms}
            onSelect={handleSelectRom}
            onToggleFavorite={handleToggleFavorite}
            onLoadMore={loadMore}
            hasMore={hasMore}
            loadingMore={loadingMore}
          />
        )}
      </div>
    </div>
  );
}
