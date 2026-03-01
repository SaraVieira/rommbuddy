import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { CacheInfo } from "../../types";
import { toast } from "sonner";
import { formatSize } from "../../utils/format";

const EVICTION_OPTIONS = [3, 7, 14, 30] as const;

export default function StorageTab() {
  const [cacheInfo, setCacheInfo] = useState<CacheInfo | null>(null);
  const [evictionDays, setEvictionDays] = useState(7);
  const [loading, setLoading] = useState(true);

  const loadCacheInfo = useCallback(async () => {
    try {
      const [info, days] = await Promise.all([
        invoke<CacheInfo>("get_cache_info"),
        invoke<number>("get_cache_eviction_days"),
      ]);
      setCacheInfo(info);
      setEvictionDays(days);
    } catch (e) {
      console.error("Failed to load cache info:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadCacheInfo();
  }, [loadCacheInfo]);

  const handleEvictionChange = async (days: number) => {
    setEvictionDays(days);
    try {
      await invoke("set_cache_eviction_days", { days });
      toast.success(`Auto-cleanup set to ${days} days`);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleClearAll = async () => {
    try {
      await invoke("clear_all_cache");
      toast.success("Cache cleared");
      loadCacheInfo();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleClearFile = async (fileName: string) => {
    try {
      await invoke("clear_cache_files", { fileNames: [fileName] });
      toast.success("File removed from cache");
      loadCacheInfo();
    } catch (e) {
      toast.error(String(e));
    }
  };

  if (loading) {
    return (
      <div className="text-center p-[40px] text-text-muted">Loading...</div>
    );
  }

  return (
    <>
      <section>
        <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">
          // Auto-Cleanup
        </h2>
        <div className="card">
          <div className="form-group">
            <label>Remove cached ROMs not played in</label>
            <div className="flex gap-md">
              {EVICTION_OPTIONS.map((days) => (
                <button
                  key={days}
                  className={`px-xl py-sm font-mono text-badge uppercase border ${
                    evictionDays === days
                      ? "border-accent text-accent bg-accent/10"
                      : "border-border text-text-muted bg-bg-elevated hover:border-border-light"
                  }`}
                  onClick={() => handleEvictionChange(days)}
                >
                  {days} days
                </button>
              ))}
            </div>
          </div>
        </div>
      </section>

      <section className="mt-3xl">
        <div className="flex items-center justify-between mb-lg">
          <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide">
            // ROM Cache
          </h2>
          <span className="font-mono text-nav text-text-secondary">
            {formatSize(cacheInfo?.total_size ?? 0)} used
          </span>
        </div>
        <div className="card">
          {!cacheInfo || cacheInfo.files.length === 0 ? (
            <div className="text-center py-xl text-text-muted font-mono text-body">
              Cache is empty
            </div>
          ) : (
            <>
              <table className="w-full border-collapse mb-xl">
                <thead>
                  <tr>
                    <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                      File
                    </th>
                    <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                      Size
                    </th>
                    <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                      Last Played
                    </th>
                    <th className="text-right p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                      Action
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {cacheInfo.files.map((file) => (
                    <tr key={file.file_name}>
                      <td className="p-md px-lg text-body text-text-primary border-b border-border truncate max-w-[300px]">
                        {file.file_name}
                      </td>
                      <td className="p-md px-lg text-body text-text-secondary border-b border-border whitespace-nowrap">
                        {formatSize(file.size)}
                      </td>
                      <td className="p-md px-lg text-body text-text-secondary border-b border-border whitespace-nowrap">
                        {file.last_played_at
                          ? new Date(file.last_played_at).toLocaleDateString()
                          : "Never"}
                      </td>
                      <td className="p-md px-lg text-body border-b border-border text-right">
                        <button
                          className="btn btn-danger btn-sm"
                          onClick={() => handleClearFile(file.file_name)}
                        >
                          Delete
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <button className="btn btn-danger" onClick={handleClearAll}>
                Clear All Cache
              </button>
            </>
          )}
        </div>
      </section>
    </>
  );
}
