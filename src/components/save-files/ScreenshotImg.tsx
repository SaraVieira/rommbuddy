import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

export default function ScreenshotImg({ path }: { path: string }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    invoke<string>("read_file_base64", { filePath: path })
      .then(setSrc)
      .catch((e) => {
        console.error("Failed to load screenshot:", e);
        toast.error(String(e));
      });
  }, [path]);
  if (!src) {
    return (
      <div className="w-16 h-12 bg-bg-elevated border border-border flex items-center justify-center shrink-0">
        <span className="font-mono text-[9px] text-text-muted">IMG</span>
      </div>
    );
  }
  return (
    <img
      src={src}
      className="w-16 h-12 object-cover border border-border shrink-0"
      alt="Save state screenshot"
    />
  );
}
