import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import type { RomWithMeta, PlatformWithCount, LibraryPage } from "../types";
import RomGrid from "../components/rom/Grid";
import RomList from "../components/rom/List";
import PlatformFilter from "../components/PlatformFilter";
import ViewToggle from "../components/ViewToggle";
import Pagination from "../components/Pagination";
import { useAppToast } from "../App";

const PAGE_SIZE = 50;

export default function Search() {
  const toast = useAppToast();
  const navigate = useNavigate();
  const inputRef = useRef<HTMLInputElement>(null);

  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [roms, setRoms] = useState<RomWithMeta[]>([]);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const [platforms, setPlatforms] = useState<PlatformWithCount[]>([]);
  const [selectedPlatform, setSelectedPlatform] = useState<number | null>(null);
  const [view, setView] = useState<"grid" | "list">("grid");
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);

  // Auto-focus on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Debounce
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(query);
      setOffset(0);
    }, 300);
    return () => clearTimeout(timer);
  }, [query]);

  // Load platforms
  useEffect(() => {
    invoke<PlatformWithCount[]>("get_platforms_with_counts")
      .then(setPlatforms)
      .catch(console.error);
  }, []);

  const doSearch = useCallback(async () => {
    if (!debouncedQuery.trim() && selectedPlatform === null) {
      setRoms([]);
      setTotal(0);
      setSearched(false);
      return;
    }
    setLoading(true);
    setSearched(true);
    try {
      const result: LibraryPage = await invoke("get_library_roms", {
        platformId: selectedPlatform,
        search: debouncedQuery || null,
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
  }, [debouncedQuery, selectedPlatform, offset, toast]);

  useEffect(() => {
    doSearch();
  }, [doSearch]);

  const handleSelectRom = (rom: RomWithMeta) => {
    navigate(`/rom/${rom.id}`, { state: { rom } });
  };

  return (
    <div className="page">
      <div className="flex flex-col gap-xs mb-xl">
        <h1 className="font-display text-page-title font-bold text-text-primary uppercase">
          Search
        </h1>
        <span className="text-nav text-text-muted">
          Search across all platforms
        </span>
      </div>
      <div className="flex items-center gap-md mb-3xl">
        <input
          ref={inputRef}
          type="text"
          className="min-w-[160px] flex-1 max-w-[400px] px-lg py-[6px] rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-body placeholder:text-text-dim focus:border-accent outline-none transition-[border-color] duration-150"
          placeholder="Search ROMs..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <PlatformFilter
          platforms={platforms}
          selected={selectedPlatform}
          onSelect={(id) => {
            setSelectedPlatform(id);
            setOffset(0);
          }}
        />
        <ViewToggle view={view} onChange={setView} />
      </div>

      {searched && !loading && roms.length > 0 && (
        <div className="text-nav text-text-muted mb-lg">
          Found {total} result{total !== 1 ? "s" : ""}
          {debouncedQuery ? <> for &ldquo;{debouncedQuery}&rdquo;</> : null}
        </div>
      )}

      <div className="mt-xl">
        {!searched ? (
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
            onToggleFavorite={(romId, fav) =>
              setRoms((prev) =>
                prev.map((r) => (r.id === romId ? { ...r, favorite: fav } : r)),
              )
            }
          />
        ) : (
          <RomList
            roms={roms}
            onSelect={handleSelectRom}
            onToggleFavorite={(romId, fav) =>
              setRoms((prev) =>
                prev.map((r) => (r.id === romId ? { ...r, favorite: fav } : r)),
              )
            }
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
