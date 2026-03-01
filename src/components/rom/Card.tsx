import { memo } from "react";
import { CheckCircle, AlertTriangle, Gamepad2, Save } from "lucide-react";
import { useAtomValue } from "jotai";
import type { RomWithMeta } from "../../types";
import { useProxiedImage } from "../../hooks/useProxiedImage";
import FavoriteButton from "../FavoriteButton";
import { romSavesAtom } from "../../store/library";

interface Props {
  rom: RomWithMeta;
  onClick: () => void;
  onToggleFavorite: (romId: number, favorite: boolean) => void;
}

export default memo(function RomCard({ rom, onClick, onToggleFavorite }: Props) {
  const coverSrc = useProxiedImage(rom.cover_url);
  const romSaves = useAtomValue(romSavesAtom);
  const hasSaves = romSaves[rom.id] ?? false;

  return (
    <div
      className="bg-bg-card border border-border rounded-none overflow-hidden cursor-pointer transition-[border-color,transform] duration-150 hover:border-border-accent-tint hover:-translate-y-0.5"
      onClick={onClick}
    >
      <div className="aspect-[3/4] bg-bg-elevated flex items-center justify-center overflow-hidden relative">
        {coverSrc ? (
          <img
            src={coverSrc}
            alt={rom.name}
            loading="lazy"
            className="w-full h-full object-cover"
          />
        ) : (
          <Gamepad2 size={40} className="text-text-dim" />
        )}
        <span className="absolute top-md left-md bg-accent text-text-on-accent font-mono text-badge font-bold uppercase tracking-wide px-sm py-xs">
          {rom.platform_slug}
        </span>
        <span className="absolute top-md right-md">
          <FavoriteButton
            romId={rom.id}
            favorite={rom.favorite}
            onToggle={onToggleFavorite}
          />
        </span>
        {rom.verification_status === "verified" && (
          <span
            className="absolute bottom-md left-md"
            title="Verified (DAT match)"
          >
            <CheckCircle size={16} className="text-accent" />
          </span>
        )}
        {rom.verification_status === "bad_dump" && (
          <span className="absolute bottom-md left-md" title="Bad dump">
            <AlertTriangle size={16} className="text-error" />
          </span>
        )}
        {hasSaves && (
          <div className="absolute top-2 right-2 flex items-center gap-xs bg-[#00FF8825] border border-[#00FF8850] px-sm py-xs">
            <Save size={10} className="text-accent" />
          </div>
        )}
      </div>
      <div className="p-lg">
        <div
          className="text-body font-medium text-text-primary overflow-hidden text-ellipsis whitespace-nowrap"
          title={rom.name}
        >
          {rom.name}
        </div>
        <div className="flex items-center justify-between mt-xs">
          <span className="text-label text-text-muted overflow-hidden text-ellipsis whitespace-nowrap">
            {rom.platform_name}
          </span>
          {rom.regions.length > 0 && (
            <span className="text-badge text-text-dim">{rom.regions[0]}</span>
          )}
        </div>
      </div>
    </div>
  );
});
