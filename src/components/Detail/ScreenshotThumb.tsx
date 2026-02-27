import { Gamepad2 } from "lucide-react";
import { useProxiedImage } from "../../hooks/useProxiedImage";

interface Props {
  url: string;
  alt: string;
  onClick: () => void;
}

export default function ScreenshotThumb({ url, alt, onClick }: Props) {
  const src = useProxiedImage(url);
  return (
    <button
      className="w-30 h-22.5 bg-bg-elevated border border-border p-0 cursor-pointer overflow-hidden hover:border-accent transition-colors shrink-0"
      onClick={onClick}
    >
      {src ? (
        <img src={src} alt={alt} className="w-full h-full object-cover" />
      ) : (
        <div className="w-full h-full flex items-center justify-center">
          <Gamepad2 size={16} className="text-border" />
        </div>
      )}
    </button>
  );
}
