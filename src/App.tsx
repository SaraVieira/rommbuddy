import { createContext, useContext, useState, useEffect } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { useAtomValue, useSetAtom } from "jotai";
import { BookOpen, Cpu, Search, Database, Settings, Heart } from "lucide-react";
import { toast } from "sonner";
import { Toaster } from "./components/ui/sonner";
import OperationProgressBanner from "./components/OperationProgressBanner";
import { useSyncState, type SyncState } from "./hooks/useSyncState";
import { useEnrichState, type EnrichState } from "./hooks/useEnrichState";
import type { SourceConfig } from "./types";
import {
  favoritesOnlyAtom,
  selectedPlatformAtom,
  searchInputAtom,
  searchAtom,
} from "./store/library";
import {
  platformCountAtom,
  romCountAtom,
  refreshPlatformsAtom,
} from "./store/platforms";

export const SyncContext = createContext<SyncState>({
  syncing: false,
  progress: null,
  startSync: async () => {},
  cancelSync: async () => {},
});
export const EnrichContext = createContext<EnrichState>({
  enriching: false,
  progress: null,
  startEnrich: async () => {},
  cancelEnrich: async () => {},
});

export function useAppSync() {
  return useContext(SyncContext);
}

export function useAppEnrich() {
  return useContext(EnrichContext);
}

const navLinkClass = ({ isActive }: { isActive: boolean }) =>
  `flex items-center gap-md px-lg py-md border-l-2 no-underline font-mono text-nav font-medium uppercase tracking-wide transition-colors ${
    isActive
      ? "border-l-accent bg-accent-tint-10 text-text-primary font-bold"
      : "border-l-transparent text-text-secondary hover:bg-accent-tint-10 hover:text-text-primary"
  }`;

export default function App() {
  const syncState = useSyncState();
  const enrichState = useEnrichState();
  const navigate = useNavigate();
  const platformCount = useAtomValue(platformCountAtom);
  const romCount = useAtomValue(romCountAtom);
  const refreshPlatforms = useSetAtom(refreshPlatformsAtom);
  const [sourceCount, setSourceCount] = useState(0);
  const [favoritesCount, setFavoritesCount] = useState(0);

  const setFavoritesOnly = useSetAtom(favoritesOnlyAtom);
  const setSelectedPlatform = useSetAtom(selectedPlatformAtom);
  const setSearchInput = useSetAtom(searchInputAtom);
  const setSearch = useSetAtom(searchAtom);

  useEffect(() => {
    (async () => {
      try {
        await refreshPlatforms();
      } catch (e) {
        console.error("Failed to load platforms:", e);
        toast.error(String(e));
      }
      try {
        const sources: SourceConfig[] = await invoke("get_sources");
        setSourceCount(sources.length);
      } catch (e) {
        console.error("Failed to load sources:", e);
        toast.error(String(e));
      }
      try {
        const count: number = await invoke("get_favorites_count");
        setFavoritesCount(count);
      } catch (e) {
        console.error("Failed to load favorites count:", e);
        toast.error(String(e));
      }
    })();
  }, []);

  const handleFavoritesClick = () => {
    setFavoritesOnly(true);
    setSelectedPlatform(null);
    setSearchInput("");
    setSearch("");
    navigate("/");
  };

  return (
    <SyncContext.Provider value={syncState}>
      <EnrichContext.Provider value={enrichState}>
        <div className="flex h-screen overflow-hidden">
          <nav className="w-sidebar bg-bg-sidebar border-r border-border flex flex-col shrink-0">
            <div
              data-tauri-drag-region
              className="pt-[38px] p-2xl px-xl border-b border-border flex items-center gap-lg"
            >
              <img
                src="/romm-buddy-icon.png"
                alt="RoMM Buddy"
                className="w-[--height-logo-mark] h-logo-mark shrink-0 rounded-lg"
              />
              <span className="font-mono text-logo font-semibold text-text-primary tracking-[1px] uppercase">
                Romm Buddy
              </span>
            </div>
            <ul className="list-none p-md flex flex-col gap-xs flex-1">
              <li>
                <NavLink to="/" end className={navLinkClass}>
                  <BookOpen size={14} />
                  <span>Library</span>
                </NavLink>
              </li>
              <li>
                <NavLink to="/platforms" className={navLinkClass}>
                  <Cpu size={14} />
                  <span>Platforms</span>
                </NavLink>
              </li>
              <li>
                <button
                  className="flex w-full items-center gap-md px-lg py-md border-l-2 border-l-transparent no-underline font-mono text-nav font-medium uppercase tracking-wide transition-colors bg-transparent cursor-pointer text-text-secondary hover:bg-accent-tint-10 hover:text-text-primary"
                  onClick={handleFavoritesClick}
                >
                  <Heart size={14} />
                  <span>Favorites</span>
                </button>
              </li>
              <li>
                <NavLink to="/search" className={navLinkClass}>
                  <Search size={14} />
                  <span>Search</span>
                </NavLink>
              </li>
              <li>
                <NavLink to="/sources" className={navLinkClass}>
                  <Database size={14} />
                  <span>Sources</span>
                </NavLink>
              </li>
              <li>
                <NavLink to="/settings" className={navLinkClass}>
                  <Settings size={14} />
                  <span>Settings</span>
                </NavLink>
              </li>
            </ul>
            <div className="border-t border-border p-xl font-mono">
              <div className="text-label text-text-muted uppercase tracking-[1px] mb-lg">
                // Collection
              </div>
              <div className="flex justify-between text-nav text-text-secondary py-sm">
                <span>Favorites</span>
                <span className="text-text-primary font-semibold">
                  {favoritesCount}
                </span>
              </div>
              <div className="flex justify-between text-nav text-text-secondary py-sm">
                <span>Platforms</span>
                <span className="text-text-primary font-semibold">
                  {platformCount}
                </span>
              </div>
              <div className="flex justify-between text-nav text-text-secondary py-sm">
                <span>ROMs</span>
                <span className="text-text-primary font-semibold">
                  {romCount}
                </span>
              </div>
              <div className="flex justify-between text-nav text-text-secondary py-sm">
                <span>Sources</span>
                <span className="text-text-primary font-semibold">
                  {sourceCount}
                </span>
              </div>
              <div
                className={`mt-lg text-badge font-semibold tracking-[1px] uppercase ${syncState.syncing || enrichState.enriching ? "text-yellow-400" : "text-accent"}`}
              >
                {syncState.syncing
                  ? "[syncing...]"
                  : enrichState.enriching
                    ? "[enriching...]"
                    : "[synced]"}
              </div>
            </div>
          </nav>
          <main className="flex-1 overflow-y-auto">
            {syncState.syncing && syncState.progress && (
              <OperationProgressBanner
                label="Syncing"
                progress={syncState.progress}
                onCancel={() => syncState.cancelSync(syncState.progress!.source_id)}
              />
            )}
            {enrichState.enriching && enrichState.progress && (
              <OperationProgressBanner
                label="Enriching"
                progress={enrichState.progress}
                color="text-yellow-400"
                onCancel={() => enrichState.cancelEnrich()}
              />
            )}
            <div className="py-5xl px-6xl pt-[38px]">
              <Outlet />
            </div>
          </main>
        </div>
        <Toaster />
      </EnrichContext.Provider>
    </SyncContext.Provider>
  );
}
