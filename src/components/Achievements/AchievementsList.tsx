import type { AchievementData } from "../../types";
import { SingleAchievement } from "./SingleAchievement";

interface Props {
  achievements: AchievementData | null;
  loading: boolean;
  error: string | null;
}

export default function AchievementsList({
  achievements,
  loading,
  error,
}: Props) {
  if (!loading && !error && !achievements) return null;

  return (
    <div className="flex flex-col gap-xl">
      <div className="flex items-center justify-between">
        <span className="font-mono text-label font-semibold text-accent tracking-[0.5px] uppercase">
          // Achievements
        </span>
        {achievements && (
          <span className="font-mono text-label text-text-secondary uppercase">
            {achievements.num_earned} / {achievements.num_achievements} unlocked
          </span>
        )}
      </div>

      {achievements && achievements.num_achievements > 0 && (
        <div className="h-1.5 bg-bg-elevated">
          <div
            className="h-full bg-accent transition-[width] duration-300"
            style={{
              width: `${(achievements.num_earned / achievements.num_achievements) * 100}%`,
            }}
          />
        </div>
      )}

      {loading ? (
        <div className="text-center p-2xl text-text-muted font-mono text-nav">
          Loading achievements...
        </div>
      ) : error ? (
        <div className="text-center p-2xl text-error font-mono text-nav">
          {error}
        </div>
      ) : achievements ? (
        <div className="max-h-75 overflow-y-auto flex flex-col gap-md">
          {achievements.achievements.map((ach) => (
            <SingleAchievement key={ach.id} achievement={ach} />
          ))}
        </div>
      ) : null}
    </div>
  );
}
