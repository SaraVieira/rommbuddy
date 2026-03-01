import { useProxiedImage } from "@/hooks/useProxiedImage";
import { RomWithMeta } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import ScreenshotThumb from "./ScreenshotThumb";
import ScreenshotModal from "./ScreenshotModal";
import { Gamepad2 } from "lucide-react";

export const LeftPanel = ({ rom }: { rom: RomWithMeta }) => {
  const coverSrc = useProxiedImage(rom?.cover_url ?? null);
  const [screenshotUrls, setScreenshotUrls] = useState<string[]>([]);
  const [screenshotModal, setScreenshotModal] = useState<string | null>(null);

  // Fetch screenshot URLs on mount
  const romId = rom?.id;
  useEffect(() => {
    if (romId == null) return;
    let cancelled = false;
    (async () => {
      try {
        const urls = await invoke<string[]>("get_rom_screenshots", {
          romId,
        });
        if (!cancelled) setScreenshotUrls(urls);
      } catch (e) {
        console.error("Failed to load screenshots:", e);
        toast.error(String(e));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [romId]);

  return (
    <div className="w-120 shrink-0 flex flex-col bg-bg-card overflow-y-auto pt-2">
      <div className="w-full min-h-130 shrink-0 bg-bg-elevated flex items-center justify-center overflow-hidden">
        {coverSrc ? (
          <img
            src={coverSrc}
            alt={rom.name}
            className="w-full h-full object-cover"
          />
        ) : (
          <Gamepad2 size={64} className="text-border" />
        )}
      </div>

      {/* Screenshot Thumbnails */}
      {screenshotUrls.length > 0 && (
        <div className="p-[16px_24px] flex flex-col gap-md">
          <span className="font-mono text-label font-semibold text-accent tracking-[0.5px] uppercase">
            // Screenshots
          </span>
          <div className="flex gap-md flex-wrap">
            {screenshotUrls.map((url, i) => (
              <ScreenshotThumb
                key={url}
                url={url}
                alt={`${rom.name} screenshot ${i + 1}`}
                onClick={() => setScreenshotModal(url)}
              />
            ))}
          </div>
        </div>
      )}
      {screenshotModal && (
        <ScreenshotModal
          url={screenshotModal}
          alt={rom.name}
          onClose={() => setScreenshotModal(null)}
        />
      )}
    </div>
  );
};
