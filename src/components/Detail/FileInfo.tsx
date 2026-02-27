import { RomWithMeta } from "@/types";

export const FileInfo = ({ rom }: { rom: RomWithMeta }) => {
  return (
    <div className="flex flex-col gap-lg bg-bg-card border border-border p-2xl">
      <span className="font-mono text-label font-semibold text-accent tracking-[0.5px]">
        // FILE INFO
      </span>
      <div className="flex justify-between">
        <span className="font-mono text-badge font-medium text-text-muted tracking-[0.5px]">
          FILENAME
        </span>
        <span className="font-mono text-label text-text-primary">
          {rom.file_name}
        </span>
      </div>
      <div className="flex justify-between">
        <span className="font-mono text-badge font-medium text-text-muted tracking-[0.5px]">
          PLATFORM
        </span>
        <span className="font-mono text-label text-text-primary">
          {rom.platform_name}
        </span>
      </div>
    </div>
  );
};
