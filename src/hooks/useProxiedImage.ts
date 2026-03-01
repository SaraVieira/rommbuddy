import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

const MAX_CACHE = 200;
const cache = new Map<string, string>();

function cacheSet(key: string, value: string) {
  if (cache.size >= MAX_CACHE) {
    const firstKey = cache.keys().next().value;
    if (firstKey !== undefined) cache.delete(firstKey);
  }
  cache.set(key, value);
}

export function useProxiedImage(url: string | null): string | null {
  const [src, setSrc] = useState<string | null>(
    url ? cache.get(url) ?? null : null
  );

  useEffect(() => {
    if (!url) {
      setSrc(null);
      return;
    }

    // Return cached immediately
    const cached = cache.get(url);
    if (cached) {
      setSrc(cached);
      return;
    }

    let cancelled = false;
    (async () => {
      try {
        const dataUrl: string = await invoke("proxy_image", { url });
        cacheSet(url, dataUrl);
        if (!cancelled) setSrc(dataUrl);
      } catch (e) {
        console.error("Failed to proxy image:", e);
        toast.error(String(e));
        if (!cancelled) setSrc(null);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [url]);

  return src;
}
