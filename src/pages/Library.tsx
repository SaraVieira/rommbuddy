import { useState, useEffect, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { useAtom } from "jotai";
import type {
  RomWithMeta,
  PlatformWithCount,
  ScanProgress,
  LibraryPage as LibraryPageType,
} from "../types";
import RomGrid from "../components/RomGrid";
import RomList from "../components/RomList";
import PlatformFilter from "../components/PlatformFilter";
import ViewToggle from "../components/ViewToggle";
import Pagination from "../components/Pagination";
import { useAppToast } from "../App";
import {
  searchInputAtom,
  searchAtom,
  selectedPlatformAtom,
  viewAtom,
  offsetAtom,
} from "../store/library";

const PAGE_SIZE = 50;

export default function Library() {
  const navigate = useNavigate();
  const toast = useAppToast();

  const [roms, setRoms] = useState<RomWithMeta[]>([]);
  const [total, setTotal] = useState(0);
  const [platforms, setPlatforms] = useState<PlatformWithCount[]>([]);
  const [loading, setLoading] = useState(false);
  const [enriching, setEnriching] = useState(false);
  const [enrichProgress, setEnrichProgress] = useState<ScanProgress | null>(
    null,
  );

  // Persistent state via jotai
  const [searchInput, setSearchInput] = useAtom(searchInputAtom);
  const [search, setSearch] = useAtom(searchAtom);
  const [selectedPlatform, setSelectedPlatform] = useAtom(selectedPlatformAtom);
  const [view, setView] = useAtom(viewAtom);
  const [offset, setOffset] = useAtom(offsetAtom);

  const loadRoms = useCallback(async () => {
    setLoading(true);
    try {
      const result: LibraryPageType = await invoke("get_library_roms", {
        platformId: selectedPlatform,
        search: search || null,
        offset,
        limit: PAGE_SIZE,
      });
      setRoms(result.roms);
      setTotal(result.total);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setLoading(false);
    }
  }, [selectedPlatform, search, offset, toast]);

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
      setOffset(0);
    }, 300);
    return () => clearTimeout(timer);
  }, [searchInput, setSearch, setOffset]);

  const handlePlatformSelect = (id: number | null) => {
    setSelectedPlatform(id);
    setOffset(0);
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
    if (enriching) return;
    setEnriching(true);
    setEnrichProgress(null);
    try {
      // Check if LaunchBox DB is imported, download + import if not
      const hasDb: boolean = await invoke("has_launchbox_db");
      if (!hasDb) {
        const dlChannel = new Channel<ScanProgress>();
        dlChannel.onmessage = (p) => setEnrichProgress(p);
        await invoke("update_launchbox_db", { channel: dlChannel });
      }

      // Run enrichment — pass current filters so only visible ROMs are enriched
      const channel = new Channel<ScanProgress>();
      channel.onmessage = (p) => setEnrichProgress(p);
      await invoke("fetch_metadata", {
        platformId: selectedPlatform,
        search: search || null,
        channel,
      });
      toast("Metadata enrichment complete!", "success");
      loadRoms();
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setEnriching(false);
      setEnrichProgress(null);
    }
  }, [enriching, selectedPlatform, search, toast, loadRoms]);

  if (total === 0 && !loading && !search && selectedPlatform === null) {
    return (
      <div className="page">
        <h1 className="font-display text-page-title font-bold text-text-primary mb-md uppercase">
          Library
        </h1>
        <div className="text-center py-[60px] px-[20px] text-text-muted">
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
    <div className="page">
      <div className="flex flex-col gap-xs mb-xl">
        <h1 className="font-display text-page-title font-bold text-text-primary uppercase">
          Library
        </h1>
        <span className="text-nav text-text-muted">
          Browse and launch your ROM collection
        </span>
      </div>
      <div className="flex items-center gap-md mb-3xl flex-wrap">
        <input
          type="text"
          className="min-w-[160px] flex-1 max-w-[320px] px-lg py-[6px] rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-body placeholder:text-text-dim focus:border-accent outline-none transition-[border-color] duration-150"
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

      {enrichProgress && (
        <div className="mb-md px-sm py-xs bg-bg-elevated border border-border rounded-none text-body text-text-muted">
          <div className="flex items-center justify-between">
            <span className="truncate mr-md">
              {enrichProgress.current_item}
            </span>
            {enrichProgress.total > 0 && (
              <span className="shrink-0">
                {enrichProgress.current} / {enrichProgress.total}
              </span>
            )}
          </div>
          {enrichProgress.total > 1 && (
            <div className="mt-xs h-[3px] bg-border">
              <div
                className="h-full bg-accent transition-[width] duration-150"
                style={{
                  width: `${(enrichProgress.current / enrichProgress.total) * 100}%`,
                }}
              />
            </div>
          )}
        </div>
      )}

      <div className="mt-xl">
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
          />
        ) : (
          <RomList
            roms={roms}
            onSelect={handleSelectRom}
            onToggleFavorite={handleToggleFavorite}
          />
        )}
      </div>

      <Pagination
        offset={offset}
        total={total}
        pageSize={PAGE_SIZE}
        onOffsetChange={setOffset}
      />
    </div>
  );
}
