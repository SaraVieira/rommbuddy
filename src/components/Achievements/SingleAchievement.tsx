import { Lock } from "lucide-react";
import { AchievementData } from "../../types";

export const SingleAchievement = ({
  achievement,
}: {
  achievement: AchievementData["achievements"][number];
}) => {
  return (
    <div
      key={achievement.id}
      className={`flex items-center gap-lg p-[12px_16px] border ${
        achievement.earned
          ? "bg-bg-card border-border"
          : "bg-bg-card border-bg-subtle opacity-50"
      }`}
    >
      <div className="w-[40px] h-[40px] shrink-0 bg-bg-elevated rounded-sm flex items-center justify-center overflow-hidden">
        {achievement.earned ? (
          <img
            src={achievement.badge_url}
            alt=""
            className="w-full h-full object-cover"
            onError={(e) => {
              (e.target as HTMLImageElement).style.display = "none";
              (e.target as HTMLImageElement).parentElement!.innerHTML =
                '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#00FF88" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6"/><path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18"/><path d="M4 22h16"/><path d="M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22"/><path d="M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22"/><path d="M18 2H6v7a6 6 0 0 0 12 0V2Z"/></svg>';
            }}
          />
        ) : (
          <Lock size={20} className="text-text-dim" />
        )}
      </div>
      <div className="flex-1 min-w-0 flex flex-col gap-sm">
        <div className="flex items-center gap-md">
          <span
            className={`font-mono text-label font-semibold truncate ${achievement.earned ? "text-text-primary" : "text-text-secondary"}`}
          >
            {achievement.title}
          </span>
          <span
            className={`font-mono text-badge shrink-0 ${achievement.earned ? "text-accent" : "text-text-dim"}`}
          >
            {achievement.points} pts
          </span>
        </div>
        <span
          className={`font-mono text-badge truncate ${achievement.earned ? "text-text-muted" : "text-text-dim"}`}
        >
          {achievement.description}
        </span>
      </div>
      <span
        className={`font-mono text-tiny font-semibold tracking-[0.5px] shrink-0 ${achievement.earned ? "text-accent" : "text-text-dim"}`}
      >
        {achievement.earned ? "EARNED" : "LOCKED"}
      </span>
    </div>
  );
};
