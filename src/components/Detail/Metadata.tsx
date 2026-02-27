import { RomWithMeta } from "@/types";
import {
  BookOpen,
  Gamepad2,
  Trophy,
  Database,
  ExternalLink,
} from "lucide-react";
import { useMemo } from "react";

export const MetadataGrid = ({ rom }: { rom: RomWithMeta }) => {
  const fileSizeMB = rom.file_size
    ? `${(rom.file_size / 1024 / 1024).toFixed(1)} MB`
    : "—";

  // Sanitize release date — reject dates with year > 2100 or malformed values
  const releaseDate = useMemo(() => {
    if (!rom.release_date) return "—";
    const year = parseInt(rom.release_date.replace(/^\+/, ""), 10);
    if (isNaN(year) || year > 2100 || year < 1950) return "—";
    return rom.release_date;
  }, [rom.release_date]);

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
    <>
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
            {rom.rating != null ? `${(rom.rating / 10).toFixed(1)} / 10` : "—"}
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
                className="font-mono text-badge font-semibold text-accent tracking-[0.5px] px-lg py-1.25 bg-accent-tint-10 border border-accent-tint-40"
              >
                {g.toUpperCase()}
              </span>
            ))}
            {rom.themes.map((t) => (
              <span
                key={`theme-${t}`}
                className="font-mono text-badge font-medium text-text-secondary tracking-[0.5px] px-lg py-1.25 border border-border"
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
    </>
  );
};
