import { RomSource, RomWithMeta } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export const Sources = ({ rom }: { rom: RomWithMeta }) => {
  const [romSources, setRomSources] = useState<RomSource[]>([]);

  // Fetch screenshot URLs and sources on mount
  useEffect(() => {
    if (!rom) return;
    let cancelled = false;

    (async () => {
      try {
        const sources = await invoke<RomSource[]>("get_rom_sources", {
          romId: rom.id,
        });
        if (!cancelled) setRomSources(sources);
      } catch {
        // ignore
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [rom?.id]);

  return romSources.length > 1 ? (
    <div className="flex flex-col gap-lg">
      <span className="font-mono text-label font-semibold text-accent tracking-[0.5px]">
        // AVAILABLE FROM {romSources.length} SOURCES
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
