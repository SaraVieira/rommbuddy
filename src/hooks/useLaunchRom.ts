import { useState } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import type { DownloadProgress } from "../types";
import { toast } from "sonner";

export function useLaunchRom(romId: number, sourceId: number) {
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);

  const launch = async (saveStateSlot?: number | null, saveStatePath?: string) => {
    setDownloading(true);
    setDownloadProgress(null);
    try {
      const channel = new Channel<DownloadProgress>();
      channel.onmessage = (progress) => setDownloadProgress(progress);
      await invoke("download_and_launch", {
        romId,
        sourceId,
        channel,
        saveStateSlot: saveStateSlot ?? null,
        saveStatePath: saveStatePath ?? null,
      });
      toast.success(saveStatePath ? "Game launched from save state!" : "Game launched!");
    } catch (e) {
      toast.error(String(e));
    } finally {
      setDownloading(false);
      setDownloadProgress(null);
    }
  };

  return { downloading, downloadProgress, launch };
}
