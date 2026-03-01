import { cn } from "@/lib/utils";

const sizeClasses = {
  section: "text-section",
  label: "text-label tracking-[0.5px]",
  sm: "text-[11px] tracking-[0.5px]",
} as const;

interface SectionHeadingProps {
  children: React.ReactNode;
  size?: keyof typeof sizeClasses;
  className?: string;
}

export default function SectionHeading({
  children,
  size = "section",
  className,
}: SectionHeadingProps) {
  const Tag = size === "section" ? "h2" : "span";
  return (
    <Tag
      className={cn(
        "font-mono font-semibold text-accent uppercase tracking-wide",
        sizeClasses[size],
        className,
      )}
    >
      // {children}
    </Tag>
  );
}
