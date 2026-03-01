import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { RomWithMeta, LibraryPage } from "../types";

const DEFAULT_PAGE_SIZE = 50;

interface UsePaginatedRomsOptions {
  platformId: number | null;
  search: string | null;
  pageSize?: number;
  enabled?: boolean;
}

interface UsePaginatedRomsResult {
  roms: RomWithMeta[];
  total: number;
  loading: boolean;
  loadingMore: boolean;
  hasMore: boolean;
  loadMore: () => Promise<void>;
  reload: () => Promise<void>;
  setRoms: React.Dispatch<React.SetStateAction<RomWithMeta[]>>;
}

export function usePaginatedRoms({
  platformId,
  search,
  pageSize = DEFAULT_PAGE_SIZE,
  enabled = true,
}: UsePaginatedRomsOptions): UsePaginatedRomsResult {
  const [roms, setRoms] = useState<RomWithMeta[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const offsetRef = useRef(0);

  const reload = useCallback(async () => {
    setLoading(true);
    offsetRef.current = 0;
    try {
      const result: LibraryPage = await invoke("get_library_roms", {
        platformId,
        search: search || null,
        offset: 0,
        limit: pageSize,
      });
      setRoms(result.roms);
      setTotal(result.total);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setLoading(false);
    }
  }, [platformId, search, pageSize]);

  const loadMore = useCallback(async () => {
    if (loadingMore) return;
    const newOffset = offsetRef.current + pageSize;
    if (newOffset >= total) return;
    setLoadingMore(true);
    offsetRef.current = newOffset;
    try {
      const result: LibraryPage = await invoke("get_library_roms", {
        platformId,
        search: search || null,
        offset: newOffset,
        limit: pageSize,
      });
      setRoms((prev) => [...prev, ...result.roms]);
      setTotal(result.total);
    } catch (e) {
      toast.error(String(e));
      offsetRef.current = newOffset - pageSize;
    } finally {
      setLoadingMore(false);
    }
  }, [loadingMore, total, platformId, search, pageSize]);

  const hasMore = offsetRef.current + pageSize < total;

  useEffect(() => {
    if (enabled) {
      reload();
    } else {
      setRoms([]);
      setTotal(0);
    }
  }, [reload, enabled]);

  return { roms, total, loading, loadingMore, hasMore, loadMore, reload, setRoms };
}
