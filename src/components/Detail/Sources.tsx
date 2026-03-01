import { RomSource, RomWithMeta } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { useAsyncInvoke } from "@/hooks/useAsyncInvoke";
import SectionHeading from "@/components/SectionHeading";

export const Sources = ({ rom }: { rom: RomWithMeta }) => {
  const romId = rom?.id;
  const { data: romSources } = useAsyncInvoke(
    () => invoke<RomSource[]>("get_rom_sources", { romId }),
    [romId],
    { enabled: romId != null },
  );

  return romSources && romSources.length > 1 ? (
    <div className="flex flex-col gap-lg">
      <SectionHeading size="label">Available from {romSources.length} sources</SectionHeading>
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
