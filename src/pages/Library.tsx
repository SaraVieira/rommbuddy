import { useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useAtom, useAtomValue } from "jotai";
import type { RomWithMeta } from "../types";
import RomGrid from "../components/rom/Grid";
import RomList from "../components/rom/List";
import PlatformFilter from "../components/PlatformFilter";
import SearchInput from "../components/SearchInput";
import ViewToggle from "../components/ViewToggle";
import { useAppEnrich } from "../App";
import {
  searchInputAtom,
  searchAtom,
  selectedPlatformAtom,
  viewAtom,
} from "../store/library";
import { platformsAtom } from "../store/platforms";
import { usePaginatedRoms } from "../hooks/usePaginatedRoms";

export default function Library() {
  const navigate = useNavigate();
  const { enriching, startEnrich } = useAppEnrich();

  const platforms = useAtomValue(platformsAtom);

  // Persistent state via jotai
  const [searchInput, setSearchInput] = useAtom(searchInputAtom);
  const [search, setSearch] = useAtom(searchAtom);
  const [selectedPlatform, setSelectedPlatform] = useAtom(selectedPlatformAtom);
  const [view, setView] = useAtom(viewAtom);

  const {
    roms, total, loading, loadingMore, hasMore, loadMore, reload, setRoms,
  } = usePaginatedRoms({
    platformId: selectedPlatform,
    search: search || null,
  });

  // Debounce search
  useEffect(() => {
    const timer = setTimeout(() => {
      setSearch(searchInput);
    }, 300);
    return () => clearTimeout(timer);
  }, [searchInput, setSearch]);

  const handlePlatformSelect = (id: number | null) => {
    setSelectedPlatform(id);
  };

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

  const handleFetchMetadata = useCallback(async () => {
    await startEnrich(selectedPlatform, search || null);
    reload();
  }, [startEnrich, selectedPlatform, search, reload]);

  if (total === 0 && !loading && !search && selectedPlatform === null) {
    return (
      <div className="page">
        <h1 className="font-display text-page-title font-bold text-text-primary mb-md uppercase">
          Library
        </h1>
        <div className="text-center py-15 px-[20px] text-text-muted">
          <p className="mb-xl text-[15px]">
            No ROMs yet. Add a source to get started.
          </p>
          <button
            className="btn btn-primary"
            onClick={() => navigate("/sources")}
          >
            Go to Sources
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex flex-col gap-xs mb-xl shrink-0">
        <h1 className="font-display text-page-title font-bold text-text-primary uppercase">
          Library
        </h1>
        <span className="text-nav text-text-muted">
          Browse and launch your ROM collection
        </span>
      </div>
      <div className="flex items-center gap-md mb-3xl flex-wrap shrink-0">
        <SearchInput
          className="min-w-40 flex-1 max-w-80"
          placeholder="Search ROMs..."
          value={searchInput}
          onChange={(e) => setSearchInput(e.target.value)}
        />
        <PlatformFilter
          platforms={platforms}
          selected={selectedPlatform}
          onSelect={handlePlatformSelect}
        />

        <ViewToggle view={view} onChange={setView} />
        <button
          className="btn btn-secondary btn-sm ml-auto"
          disabled={enriching}
          onClick={handleFetchMetadata}
        >
          {enriching ? "Enriching..." : "Fetch Metadata"}
        </button>
      </div>

      <div className="flex-1 min-h-0">
        {loading ? (
          <div className="text-center p-[40px] text-text-muted">Loading...</div>
        ) : roms.length === 0 ? (
          <div className="text-center p-[40px] text-text-muted">
            No ROMs found.
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
