import { useState, useEffect, type DependencyList } from "react";

/**
 * Wraps an async function in a useEffect with automatic cancellation.
 * Replaces the `let cancelled = false` + async IIFE + cleanup pattern.
 */
export function useAsyncInvoke<T>(
  fn: () => Promise<T>,
  deps: DependencyList,
  options?: { enabled?: boolean },
): { data: T | null; loading: boolean; error: string | null } {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const enabled = options?.enabled ?? true;

  useEffect(() => {
    if (!enabled) return;

    let cancelled = false;
    setLoading(true);
    setError(null);

    (async () => {
      try {
        const result = await fn();
        if (!cancelled) setData(result);
      } catch (e) {
        if (!cancelled) setError(String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [...deps, enabled]); // eslint-disable-line react-hooks/exhaustive-deps

  return { data, loading, error };
}
