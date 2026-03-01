import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { useAtom } from "jotai";
import type {
  RomWithMeta,
  PlatformWithCount,
  LibraryPage as LibraryPageType,
} from "../types";
import RomGrid from "../components/rom/Grid";
import RomList from "../components/rom/List";
import PlatformFilter from "../components/PlatformFilter";
import ViewToggle from "../components/ViewToggle";
import { useAppToast, useAppEnrich } from "../App";
import {
  searchInputAtom,
  searchAtom,
  selectedPlatformAtom,
  viewAtom,
} from "../store/library";

const PAGE_SIZE = 50;

export default function Library() {
  const navigate = useNavigate();
  const toast = useAppToast();
  const { enriching, startEnrich } = useAppEnrich();

  const [roms, setRoms] = useState<RomWithMeta[]>([]);
  const [total, setTotal] = useState(0);
  const [platforms, setPlatforms] = useState<PlatformWithCount[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);

  // Local offset — internal to loading logic
  const offsetRef = useRef(0);

  // Persistent state via jotai
  const [searchInput, setSearchInput] = useAtom(searchInputAtom);
  const [search, setSearch] = useAtom(searchAtom);
  const [selectedPlatform, setSelectedPlatform] = useAtom(selectedPlatformAtom);
  const [view, setView] = useAtom(viewAtom);

  const loadRoms = useCallback(async () => {
    setLoading(true);
    offsetRef.current = 0;
    try {
      const result: LibraryPageType = await invoke("get_library_roms", {
        platformId: selectedPlatform,
        search: search || null,
        offset: 0,
        limit: PAGE_SIZE,
      });
      setRoms(result.roms);
      setTotal(result.total);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setLoading(false);
    }
  }, [selectedPlatform, search, toast]);

  const loadMore = useCallback(async () => {
    if (loadingMore) return;
    const newOffset = offsetRef.current + PAGE_SIZE;
    if (newOffset >= total) return;
    setLoadingMore(true);
    offsetRef.current = newOffset;
    try {
      const result: LibraryPageType = await invoke("get_library_roms", {
        platformId: selectedPlatform,
        search: search || null,
        offset: newOffset,
        limit: PAGE_SIZE,
      });
      setRoms((prev) => [...prev, ...result.roms]);
      setTotal(result.total);
    } catch (e) {
      toast(String(e), "error");
      // Roll back offset on failure
      offsetRef.current = newOffset - PAGE_SIZE;
    } finally {
      setLoadingMore(false);
    }
  }, [loadingMore, total, selectedPlatform, search, toast]);

  const hasMore = offsetRef.current + PAGE_SIZE < total;

  const loadPlatforms = useCallback(async () => {
    try {
      const result: PlatformWithCount[] = await invoke(
        "get_platforms_with_counts",
      );
      setPlatforms(result);
    } catch (e) {
      console.error("Failed to load platforms:", e);
    }
  }, []);

  useEffect(() => {
    loadPlatforms();
  }, [loadPlatforms]);

  useEffect(() => {
    loadRoms();
  }, [loadRoms]);

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
    loadRoms();
  }, [startEnrich, selectedPlatform, search, loadRoms]);

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
        <input
          type="text"
          className="min-w-40 flex-1 max-w-80 px-lg py-1.5 rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-body placeholder:text-text-dim focus:border-accent outline-none transition-[border-color] duration-150"
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
