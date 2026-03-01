import { useState } from "react";
import { Lock, Trophy } from "lucide-react";
import { AchievementData } from "../../types";

export const SingleAchievement = ({
  achievement,
}: {
  achievement: AchievementData["achievements"][number];
}) => {
  const [imgError, setImgError] = useState(false);

  return (
    <div
      className={`flex items-center gap-lg p-[12px_16px] border ${
        achievement.earned
          ? "bg-bg-card border-border"
          : "bg-bg-card border-bg-subtle opacity-50"
      }`}
    >
      <div className="w-[40px] h-[40px] shrink-0 bg-bg-elevated rounded-sm flex items-center justify-center overflow-hidden">
        {achievement.earned ? (
          imgError ? (
            <Trophy size={20} className="text-accent" />
          ) : (
            <img
              src={achievement.badge_url}
              alt=""
              className="w-full h-full object-cover"
              onError={() => setImgError(true)}
            />
          )
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
        className={`font-mono text-tiny font-semibold tracking-[0.5px] shrink-0 uppercase ${achievement.earned ? "text-accent" : "text-text-dim"}`}
      >
        {achievement.earned ? "Earned" : "Locked"}
      </span>
    </div>
  );
};
