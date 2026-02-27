import { CheckCircle, AlertTriangle } from "lucide-react";
import type { RomWithMeta } from "../types";
import { useProxiedImage } from "../hooks/useProxiedImage";
import FavoriteButton from "./FavoriteButton";

interface Props {
  rom: RomWithMeta;
  onClick: () => void;
  onToggleFavorite: (romId: number, favorite: boolean) => void;
}

function formatSize(bytes: number | null): string {
  if (bytes == null) return "\u2014";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export default function RomRow({ rom, onClick, onToggleFavorite }: Props) {
  const coverSrc = useProxiedImage(rom.cover_url);

  return (
    <tr
      className="cursor-pointer transition-colors duration-100 hover:bg-bg-elevated [&>td]:py-1.5 [&>td]:px-lg [&>td]:text-body [&>td]:text-text-primary [&>td]:border-b [&>td]:border-border [&>td]:align-middle"
      onClick={onClick}
    >
      <td style={{ width: 32 }}>
        <FavoriteButton romId={rom.id} favorite={rom.favorite} onToggle={onToggleFavorite} size={14} />
      </td>
      <td>
        {coverSrc ? (
          <img src={coverSrc} alt="" width={32} height={42} className="rounded-none object-cover block" />
        ) : (
          <div className="w-8 h-[42px] bg-bg-elevated rounded-none flex items-center justify-center text-sm text-text-dim font-bold">
            {rom.name.charAt(0)}
          </div>
        )}
      </td>
      <td>{rom.name}</td>
      <td className="!text-text-muted">{rom.platform_name}</td>
      <td className="!text-text-muted !text-nav">
        {rom.regions.length > 0 ? rom.regions.join(", ") : "\u2014"}
      </td>
      <td className="!text-text-muted !text-nav">{formatSize(rom.file_size)}</td>
      <td style={{ width: 28 }}>
        {rom.verification_status === "verified" && (
          <span title="Verified"><CheckCircle size={14} className="text-accent" /></span>
        )}
        {rom.verification_status === "bad_dump" && (
          <span title="Bad dump"><AlertTriangle size={14} className="text-error" /></span>
        )}
      </td>
    </tr>
  );
}
