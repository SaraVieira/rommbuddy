import { useState, useEffect } from "react";
import { useLocation, useNavigate, useParams } from "react-router-dom";
import { invoke, Channel } from "@tauri-apps/api/core";
import {
  ArrowLeft,
  Play,
  Download,
  RefreshCw,
  BookOpen,
  Gamepad2,
  Trophy,
  Database,
  ExternalLink,
} from "lucide-react";
import type {
  RomWithMeta,
  RomSource,
  DownloadProgress,
  AchievementData,
  RaCredentials,
} from "../types";
import { useProxiedImage } from "../hooks/useProxiedImage";
import { useAppToast } from "../App";
import ProgressBar from "../components/ProgressBar";
import FavoriteButton from "../components/FavoriteButton";
import ScreenshotThumb from "../components/ScreenshotThumb";
import ScreenshotModal from "../components/ScreenshotModal";
import AchievementsList from "../components/Achievements/AchievementsList";
import { SaveFiles } from "../components/SaveFiles";

export default function RomDetailPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const toast = useAppToast();
  const initialRom = location.state?.rom as RomWithMeta | undefined;

  const [rom, setRom] = useState<RomWithMeta | undefined>(initialRom);
  const [loadingRom, setLoadingRom] = useState(false);

  // Fetch ROM by ID if not passed via route state
  useEffect(() => {
    if (initialRom || !id) return;
    let cancelled = false;
    setLoadingRom(true);
    (async () => {
      try {
        const fetched = await invoke<RomWithMeta>("get_rom", { romId: Number(id) });
        if (!cancelled) setRom(fetched);
      } catch {
        // ROM not found — will show fallback
      } finally {
        if (!cancelled) setLoadingRom(false);
      }
    })();
    return () => { cancelled = true; };
  }, [id, initialRom]);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] =
    useState<DownloadProgress | null>(null);
  const [enriching, setEnriching] = useState(false);

  const coverSrc = useProxiedImage(rom?.cover_url ?? null);
  const [screenshotUrls, setScreenshotUrls] = useState<string[]>([]);
  const [screenshotModal, setScreenshotModal] = useState<string | null>(null);
  const [romSources, setRomSources] = useState<RomSource[]>([]);

  const isLocal = rom?.source_type === "local";

  // Fetch screenshot URLs and sources on mount
  useEffect(() => {
    if (!rom) return;
    let cancelled = false;
    (async () => {
      try {
        const urls = await invoke<string[]>("get_rom_screenshots", {
          romId: rom.id,
        });
        if (!cancelled) setScreenshotUrls(urls);
      } catch {
        // ignore
      }
    })();
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

  // Achievements
  const [achievements, setAchievements] = useState<AchievementData | null>(
    null,
  );
  const [achievementsLoading, setAchievementsLoading] = useState(false);
  const [achievementsError, setAchievementsError] = useState<string | null>(
    null,
  );

  useEffect(() => {
    if (!rom) return;

    let cancelled = false;
    setAchievementsLoading(true);
    setAchievementsError(null);

    (async () => {
      try {
        const creds = await invoke<RaCredentials | null>("get_ra_credentials");
        if (!creds || cancelled) {
          setAchievementsLoading(false);
          return;
        }
        const data = await invoke<AchievementData>("get_achievements", {
          romId: rom.id,
        });
        if (!cancelled) setAchievements(data);
      } catch (e) {
        if (!cancelled) {
          const msg = String(e);
          if (!msg.includes("No RetroAchievements game found")) {
            setAchievementsError(msg);
          }
        }
      } finally {
        if (!cancelled) setAchievementsLoading(false);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [rom?.id]);

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

  const handlePlay = async () => {
    setDownloading(true);
    setDownloadProgress(null);
    try {
      const channel = new Channel<DownloadProgress>();
      channel.onmessage = (progress) => setDownloadProgress(progress);
      await invoke("download_and_launch", {
        romId: rom.id,
        sourceId: rom.source_id,
        channel,
      });
      toast("Game launched!", "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setDownloading(false);
      setDownloadProgress(null);
    }
  };

  const handlePlayFromSave = async (slot: number | null, filePath: string) => {
    setDownloading(true);
    setDownloadProgress(null);
    try {
      const channel = new Channel<DownloadProgress>();
      channel.onmessage = (progress) => setDownloadProgress(progress);
      await invoke("download_and_launch", {
        romId: rom.id,
        sourceId: rom.source_id,
        channel,
        saveStateSlot: slot,
        saveStatePath: filePath,
      });
      toast("Game launched from save state!", "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setDownloading(false);
      setDownloadProgress(null);
    }
  };

  const fileSizeMB = rom.file_size
    ? `${(rom.file_size / 1024 / 1024).toFixed(1)} MB`
    : "—";

  // Sanitize release date — reject dates with year > 2100 or malformed values
  const releaseDate = (() => {
    if (!rom.release_date) return "—";
    const year = parseInt(rom.release_date.replace(/^\+/, ""), 10);
    if (isNaN(year) || year > 2100 || year < 1950) return "—";
    return rom.release_date;
  })();

  const externalLinks = [
    rom.wikipedia_url && {
      label: "WIKIPEDIA",
      url: rom.wikipedia_url,
      icon: BookOpen,
    },
    rom.igdb_id && {
      label: "IGDB",
      url: `https://www.igdb.com/games/${rom.name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`,
      icon: Gamepad2,
    },
    rom.retroachievements_game_id && {
      label: "RETROACHIEVEMENTS",
      url: `https://retroachievements.org/game/${rom.retroachievements_game_id}`,
      icon: Trophy,
    },
    rom.thegamesdb_game_id && {
      label: "THEGAMESDB",
      url: `https://thegamesdb.net/game.php?id=${rom.thegamesdb_game_id}`,
      icon: Database,
    },
  ].filter(Boolean) as { label: string; url: string; icon: typeof BookOpen }[];

  return (
    <div className="flex h-full">
      {/* Screenshot Modal */}
      {screenshotModal && (
        <ScreenshotModal
          url={screenshotModal}
          alt={rom.name}
          onClose={() => setScreenshotModal(null)}
        />
      )}

      {/* Left Panel — Cover + Screenshots */}
      <div className="w-[480px] shrink-0 flex flex-col bg-bg-card overflow-y-auto">
        <div className="w-full h-[520px] shrink-0 bg-bg-elevated flex items-center justify-center overflow-hidden">
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
            <span className="font-mono text-label font-semibold text-accent tracking-[0.5px]">
              // SCREENSHOTS
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
      </div>

      {/* Right Panel — Details (scrollable) */}
      <div className="flex-1 min-w-0 overflow-y-auto flex flex-col gap-3xl p-[32px_40px]">
        {/* Back + Action Buttons Row */}
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
                onClick={handlePlay}
                disabled={downloading}
              >
                <Play size={16} />
                LAUNCH
              </button>
              {!isLocal && (
                <button
                  className="btn btn-secondary flex items-center gap-lg"
                  onClick={handlePlay}
                  disabled={downloading}
                >
                  <Download size={16} />
                  DOWNLOAD
                </button>
              )}
            </div>
          )}
        </div>

        {/* Title */}
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

        {/* Metadata Grid */}
        <div className="flex bg-bg-card border border-border">
          <div className="flex-1 p-xl flex flex-col gap-sm">
            <span className="font-mono text-badge font-semibold text-text-muted tracking-[0.5px]">
              RELEASE
            </span>
            <span className="font-mono text-body font-medium text-text-primary">
              {releaseDate}
            </span>
          </div>
          <div className="flex-1 p-xl flex flex-col gap-sm border-l border-border">
            <span className="font-mono text-badge font-semibold text-text-muted tracking-[0.5px]">
              RATING
            </span>
            <span className="font-mono text-body font-semibold text-accent">
              {rom.rating != null
                ? `${(rom.rating / 10).toFixed(1)} / 10`
                : "—"}
            </span>
          </div>
          <div className="flex-1 p-xl flex flex-col gap-sm border-l border-border">
            <span className="font-mono text-badge font-semibold text-text-muted tracking-[0.5px]">
              FILE SIZE
            </span>
            <span className="font-mono text-body font-medium text-text-primary">
              {fileSizeMB}
            </span>
          </div>
          <div className="flex-1 p-xl flex flex-col gap-sm border-l border-border">
            <span className="font-mono text-badge font-semibold text-text-muted tracking-[0.5px]">
              SOURCE
            </span>
            <span className="font-mono text-body font-medium text-accent">
              #{rom.source_id}
            </span>
          </div>
        </div>

        {/* Description */}
        {rom.description && (
          <div className="flex flex-col gap-lg">
            <span className="font-mono text-label font-semibold text-accent tracking-[0.5px]">
              // DESCRIPTION
            </span>
            <p className="font-mono text-nav text-text-secondary leading-[1.6] m-0">
              {rom.description}
            </p>
          </div>
        )}

        {/* Genres & Themes */}
        {(rom.genres.length > 0 || rom.themes.length > 0) && (
          <div className="flex flex-col gap-lg">
            <span className="font-mono text-label font-semibold text-accent tracking-[0.5px]">
              // GENRES & THEMES
            </span>
            <div className="flex flex-wrap gap-md">
              {rom.genres.map((g) => (
                <span
                  key={`genre-${g}`}
                  className="font-mono text-badge font-semibold text-accent tracking-[0.5px] px-lg py-[5px] bg-accent-tint-10 border border-accent-tint-40"
                >
                  {g.toUpperCase()}
                </span>
              ))}
              {rom.themes.map((t) => (
                <span
                  key={`theme-${t}`}
                  className="font-mono text-badge font-medium text-text-secondary tracking-[0.5px] px-lg py-[5px] border border-border"
                >
                  {t.toUpperCase()}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Developer / Publisher */}
        {(rom.developer || rom.publisher) && (
          <div className="flex gap-3xl">
            {rom.developer && (
              <div className="flex flex-col gap-sm">
                <span className="font-mono text-badge font-semibold text-text-muted tracking-[0.5px]">
                  DEVELOPER
                </span>
                <span className="font-mono text-nav font-medium text-text-primary">
                  {rom.developer}
                </span>
              </div>
            )}
            {rom.publisher && (
              <div className="flex flex-col gap-sm">
                <span className="font-mono text-badge font-semibold text-text-muted tracking-[0.5px]">
                  PUBLISHER
                </span>
                <span className="font-mono text-nav font-medium text-text-primary">
                  {rom.publisher}
                </span>
              </div>
            )}
          </div>
        )}

        {/* External Links */}
        {externalLinks.length > 0 && (
          <div className="flex flex-col gap-lg">
            <span className="font-mono text-label font-semibold text-accent tracking-[0.5px]">
              // EXTERNAL LINKS
            </span>
            <div className="flex flex-wrap gap-lg">
              {externalLinks.map((link) => (
                <a
                  key={link.label}
                  href={link.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-md bg-bg-elevated border border-border px-xl py-md no-underline hover:border-border-light transition-colors"
                >
                  <link.icon size={14} className="text-text-secondary" />
                  <span className="font-mono text-badge font-semibold text-text-primary tracking-[0.5px]">
                    {link.label}
                  </span>
                  <ExternalLink size={10} className="text-text-muted" />
                </a>
              ))}
            </div>
          </div>
        )}

        {/* Achievements */}
        <AchievementsList
          achievements={achievements}
          loading={achievementsLoading}
          error={achievementsError}
        />

        {/* Save Files */}
        {rom && <SaveFiles romId={rom.id} onLaunchSaveState={handlePlayFromSave} />}

        {/* File Info */}
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

        {/* Sources */}
        {romSources.length > 1 && (
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
                    <span className="font-mono text-badge text-text-muted truncate max-w-[300px]">
                      {src.file_name}
                    </span>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Refetch Metadata */}
        <button
          className="flex items-center gap-md bg-bg-card border border-border px-xl py-lg font-mono text-label font-semibold text-text-primary cursor-pointer hover:border-border-light transition-colors self-start"
          disabled={enriching}
          onClick={async () => {
            if (!rom) return;
            setEnriching(true);
            try {
              const updated = await invoke<RomWithMeta>("enrich_single_rom", {
                romId: rom.id,
              });
              setRom(updated);
              setScreenshotUrls(updated.screenshot_urls);
              toast("Metadata refreshed", "success");
            } catch (e) {
              toast(String(e), "error");
            } finally {
              setEnriching(false);
            }
          }}
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
