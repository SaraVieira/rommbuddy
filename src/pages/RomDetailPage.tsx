import { useState, useEffect } from "react";
import { useLocation, useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, Play, Download, RefreshCw } from "lucide-react";
import type { RomWithMeta } from "../types";
import { useAppToast } from "../App";
import ProgressBar from "../components/ProgressBar";
import FavoriteButton from "../components/FavoriteButton";
import AchievementsList from "../components/achievements/AchievementsList";
import { SaveFiles } from "../components/save-files";
import { MetadataGrid } from "@/components/detail/Metadata";
import { FileInfo } from "@/components/detail/FileInfo";
import { LeftPanel } from "@/components/detail/Left";
import { useLaunchRom } from "../hooks/useLaunchRom";
import { useAchievements } from "../hooks/useAchievements";

export default function RomDetailPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const toast = useAppToast();
  const initialRom = location.state?.rom as RomWithMeta | undefined;

  const [rom, setRom] = useState<RomWithMeta | undefined>(initialRom);
  const [loadingRom, setLoadingRom] = useState(false);
  const [enriching, setEnriching] = useState(false);

  useEffect(() => {
    if (initialRom || !id) return;
    let cancelled = false;
    setLoadingRom(true);
    (async () => {
      try {
        const fetched = await invoke<RomWithMeta>("get_rom", {
          romId: Number(id),
        });
        if (!cancelled) setRom(fetched);
      } catch {
        // ROM not found
      } finally {
        if (!cancelled) setLoadingRom(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [id, initialRom]);

  const { downloading, downloadProgress, launch } = useLaunchRom(
    rom?.id ?? 0,
    rom?.source_id ?? 0,
  );
  const {
    achievements,
    loading: achievementsLoading,
    error: achievementsError,
  } = useAchievements(rom?.id);

  const isLocal = rom?.source_type === "local";

  if (!rom) {
    return (
      <div className="page">
        <button
          className="flex items-center gap-md text-text-secondary font-mono text-label uppercase tracking-[0.5px] cursor-pointer bg-transparent border-none hover:underline mb-3xl"
          onClick={() => navigate("/")}
        >
          <ArrowLeft size={16} />
          BACK TO LIBRARY
        </button>
        <div className="text-center py-7xl text-text-muted">
          {loadingRom ? "Loading..." : "ROM not found."}
        </div>
      </div>
    );
  }

  const handleEnrich = async () => {
    setEnriching(true);
    try {
      const updated = await invoke<RomWithMeta>("enrich_single_rom", {
        romId: rom.id,
      });
      setRom(updated);
      toast("Metadata refreshed", "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setEnriching(false);
    }
  };

  return (
    <div className="flex h-full">
      <LeftPanel rom={rom} />

      <div className="flex-1 min-w-0 overflow-y-auto flex flex-col gap-3xl p-[32px_40px]">
        <div className="flex items-center justify-between">
          <button
            className="flex items-center gap-md text-text-secondary font-mono text-label uppercase tracking-[0.5px] cursor-pointer bg-transparent border-none hover:text-text-primary"
            onClick={() => navigate(-1)}
          >
            <ArrowLeft size={16} />
            BACK TO LIBRARY
          </button>

          {downloading && downloadProgress ? (
            <div className="w-[240px] shrink-0">
              <ProgressBar
                current={downloadProgress.downloaded_bytes}
                total={downloadProgress.total_bytes}
                label={downloadProgress.status}
              />
            </div>
          ) : (
            <div className="flex gap-md shrink-0">
              <button
                className="btn btn-primary flex items-center gap-lg"
                onClick={() => launch()}
                disabled={downloading}
              >
                <Play size={16} />
                LAUNCH
              </button>
              {!isLocal && (
                <button
                  className="btn btn-secondary flex items-center gap-lg"
                  onClick={() => launch()}
                  disabled={downloading}
                >
                  <Download size={16} />
                  DOWNLOAD
                </button>
              )}
            </div>
          )}
        </div>

        <div className="flex flex-col gap-lg">
          <div className="flex items-center gap-lg">
            <h1 className="font-display text-[28px] font-bold text-text-primary tracking-[-0.5px] m-0">
              {rom.name}
            </h1>
            <FavoriteButton
              romId={rom.id}
              favorite={rom.favorite}
              onToggle={(_, fav) =>
                setRom((prev) => (prev ? { ...prev, favorite: fav } : prev))
              }
              size={22}
            />
          </div>
          <div className="flex items-center gap-md flex-wrap">
            <span className="bg-accent-tint-20 text-accent font-mono text-badge font-bold tracking-[0.5px] px-lg py-sm">
              {rom.platform_slug.toUpperCase()}
            </span>
            {rom.regions.map((r) => (
              <span
                key={r}
                className="text-text-secondary font-mono text-badge font-medium px-lg py-sm border border-border"
              >
                {r}
              </span>
            ))}
            {rom.languages.length > 0 && (
              <>
                <span className="text-text-dim font-mono text-nav font-bold">
                  ·
                </span>
                {rom.languages.map((l) => (
                  <span
                    key={l}
                    className="text-text-secondary font-mono text-badge font-medium px-lg py-sm border border-border"
                  >
                    {l}
                  </span>
                ))}
              </>
            )}
          </div>
        </div>

        <MetadataGrid rom={rom} />
        <AchievementsList
          achievements={achievements}
          loading={achievementsLoading}
          error={achievementsError}
        />
        <SaveFiles
          romId={rom.id}
          onLaunchSaveState={(slot, path) => launch(slot, path)}
        />
        <FileInfo rom={rom} />

        <button
          className="flex items-center gap-md bg-bg-card border border-border px-xl py-lg font-mono text-label font-semibold text-text-primary cursor-pointer hover:border-border-light transition-colors self-start"
          disabled={enriching}
          onClick={handleEnrich}
        >
          <RefreshCw
            size={14}
            className={`text-text-secondary ${enriching ? "animate-spin" : ""}`}
          />
          {enriching ? "FETCHING..." : "REFETCH METADATA"}
        </button>
      </div>
    </div>
  );
}
