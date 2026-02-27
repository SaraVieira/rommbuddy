import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AchievementData, RaCredentials } from "../types";

export function useAchievements(romId: number | undefined) {
  const [achievements, setAchievements] = useState<AchievementData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!romId) return;

    let cancelled = false;
    setLoading(true);
    setError(null);

    (async () => {
      try {
        const creds = await invoke<RaCredentials | null>("get_ra_credentials");
        if (!creds || cancelled) {
          setLoading(false);
          return;
        }
        const data = await invoke<AchievementData>("get_achievements", { romId });
        if (!cancelled) setAchievements(data);
      } catch (e) {
        if (!cancelled) {
          const msg = String(e);
          if (!msg.includes("No RetroAchievements game found")) {
            setError(msg);
          }
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [romId]);

  return { achievements, loading, error };
}
