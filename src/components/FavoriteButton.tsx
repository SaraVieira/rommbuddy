import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Heart } from "lucide-react";

interface Props {
  romId: number;
  favorite: boolean;
  onToggle: (romId: number, favorite: boolean) => void;
  size?: number;
}

export default function FavoriteButton({
  romId,
  favorite,
  onToggle,
  size = 16,
}: Props) {
  const [pending, setPending] = useState(false);

  const handleClick = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (pending) return;

    const newValue = !favorite;
    // Optimistic update
    onToggle(romId, newValue);
    setPending(true);

    try {
      await invoke("toggle_favorite", { romId, favorite: newValue });
    } catch {
      // Revert on failure
      onToggle(romId, favorite);
    } finally {
      setPending(false);
    }
  };

  return (
    <button
      className="bg-transparent border-none cursor-pointer p-0 flex items-center justify-center transition-transform duration-100 hover:scale-110"
      onClick={handleClick}
      title={favorite ? "Remove from favorites" : "Add to favorites"}
    >
      <Heart
        size={size}
        className={
          favorite
            ? "fill-red-500 text-red-500"
            : "fill-transparent text-[#6a6a6a] hover:text-[#8a8a8a]"
        }
      />
    </button>
  );
}
