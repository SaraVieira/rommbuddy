import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const cache = new Map<string, string>();

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
        cache.set(url, dataUrl);
        if (!cancelled) setSrc(dataUrl);
      } catch {
        if (!cancelled) setSrc(null);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [url]);

  return src;
}
