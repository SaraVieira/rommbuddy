import { RomSource, RomWithMeta } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { toast } from "sonner";

export const Sources = ({ rom }: { rom: RomWithMeta }) => {
  const [romSources, setRomSources] = useState<RomSource[]>([]);

  // Fetch sources on mount
  const romId = rom?.id;
  useEffect(() => {
    if (romId == null) return;
    let cancelled = false;

    (async () => {
      try {
        const sources = await invoke<RomSource[]>("get_rom_sources", {
          romId,
        });
        if (!cancelled) setRomSources(sources);
      } catch (e) {
        console.error("Failed to load ROM sources:", e);
        toast.error(String(e));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [romId]);

  return romSources.length > 1 ? (
    <div className="flex flex-col gap-lg">
      <span className="font-mono text-label font-semibold text-accent tracking-[0.5px] uppercase">
        // Available from {romSources.length} sources
      </span>
      <div className="flex flex-col gap-md">
        {romSources.map((src) => (
          <div
            key={src.source_id}
            className="flex items-center justify-between bg-bg-card border border-border px-xl py-md"
          >
            <div className="flex items-center gap-lg">
              <span className="font-mono text-badge font-bold text-accent tracking-[0.5px] uppercase">
                {src.source_type}
              </span>
              <span className="font-mono text-label text-text-primary">
                {src.source_name}
              </span>
            </div>
            {src.file_name && (
              <span className="font-mono text-badge text-text-muted truncate max-w-75">
                {src.file_name}
              </span>
            )}
          </div>
        ))}
      </div>
    </div>
  ) : null;
};
