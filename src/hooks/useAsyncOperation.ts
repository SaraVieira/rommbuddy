import { useState, useCallback, useRef } from "react";
import { Channel } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { ScanProgress } from "../types";

export interface AsyncOperationState<TStartArgs extends unknown[], TCancelArgs extends unknown[] = TStartArgs> {
  running: boolean;
  progress: ScanProgress | null;
  start: (...args: TStartArgs) => Promise<void>;
  cancel: (...args: TCancelArgs) => Promise<void>;
}

export function useAsyncOperation<TStartArgs extends unknown[], TCancelArgs extends unknown[] = TStartArgs>(config: {
  run: (setProgress: (p: ScanProgress) => void, ...args: TStartArgs) => Promise<void>;
  cancel: (...args: TCancelArgs) => Promise<void>;
  successMessage: string;
  errorPrefix: string;
  onComplete?: () => void;
}): AsyncOperationState<TStartArgs, TCancelArgs> {
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const runningRef = useRef(false);

  const start = useCallback(
    async (...args: TStartArgs) => {
      if (runningRef.current) return;
      runningRef.current = true;
      setRunning(true);
      setProgress(null);
      try {
        await config.run(setProgress, ...args);
        toast.success(config.successMessage);
        config.onComplete?.();
      } catch (e) {
        toast.error(`${config.errorPrefix}: ${e}`);
      } finally {
        runningRef.current = false;
        setRunning(false);
        setProgress(null);
      }
    },
    [config],
  );

  const cancel = useCallback(
    async (...args: TCancelArgs) => {
      await config.cancel(...args);
    },
    [config],
  );

  return { running, progress, start, cancel };
}

export function createProgressChannel(
  setProgress: (p: ScanProgress) => void,
): Channel<ScanProgress> {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = (p) => setProgress(p);
  return channel;
}
