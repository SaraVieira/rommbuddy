import { useState } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import type { DownloadProgress } from "../types";
import { useAppToast } from "../App";

export function useLaunchRom(romId: number, sourceId: number) {
  const toast = useAppToast();
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
      toast(saveStatePath ? "Game launched from save state!" : "Game launched!", "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setDownloading(false);
      setDownloadProgress(null);
    }
  };

  return { downloading, downloadProgress, launch };
}
