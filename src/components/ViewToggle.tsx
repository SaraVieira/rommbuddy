interface Props {
  view: "grid" | "list";
  onChange: (view: "grid" | "list") => void;
}

export default function ViewToggle({ view, onChange }: Props) {
  return (
    <div className="flex border border-border rounded-none overflow-hidden">
      <button
        className={`py-1.5 px-2.5 bg-transparent border-none cursor-pointer text-base transition-all duration-150 ${
          view === "grid"
            ? "bg-bg-elevated text-accent"
            : "text-text-muted hover:text-text-primary"
        }`}
        onClick={() => onChange("grid")}
        title="Grid view"
      >
        &#9638;
      </button>
      <button
        className={`py-1.5 px-2.5 bg-transparent border-none border-l border-l-border cursor-pointer text-base transition-all duration-150 ${
          view === "list"
            ? "bg-bg-elevated text-accent"
            : "text-text-muted hover:text-text-primary"
        }`}
        onClick={() => onChange("list")}
        title="List view"
      >
        &#9776;
      </button>
    </div>
  );
}
