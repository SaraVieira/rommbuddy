import { useEffect } from "react";
import { X } from "lucide-react";
import { useProxiedImage } from "../../hooks/useProxiedImage";

interface Props {
  url: string;
  alt: string;
  onClose: () => void;
}

export default function ScreenshotModal({ url, alt, onClose }: Props) {
  const src = useProxiedImage(url);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  return (
    <div
      className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center cursor-pointer"
      onClick={onClose}
    >
      <button
        className="absolute top-xl right-xl bg-transparent border-none cursor-pointer text-white/60 hover:text-white"
        onClick={onClose}
      >
        <X size={24} />
      </button>
      {src ? (
        <img
          src={src}
          alt={`${alt} screenshot`}
          className="max-w-[90vw] max-h-[90vh] object-contain"
          onClick={(e) => e.stopPropagation()}
        />
      ) : (
        <div className="text-white/60 font-mono text-body">Loading...</div>
      )}
    </div>
  );
}
