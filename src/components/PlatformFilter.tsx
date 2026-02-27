import { useState } from "react";
import { Check, ChevronsUpDown } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import type { PlatformWithCount } from "../types";

interface Props {
  platforms: PlatformWithCount[];
  selected: number | null;
  onSelect: (id: number | null) => void;
}

export default function PlatformFilter({
  platforms,
  selected,
  onSelect,
}: Props) {
  const [open, setOpen] = useState(false);
  const total = platforms.reduce((sum, p) => sum + p.rom_count, 0);

  const selectedPlatform = platforms.find((p) => p.id === selected);
  const label = selectedPlatform
    ? `${selectedPlatform.name} (${selectedPlatform.rom_count})`
    : `All Platforms (${total})`;

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          role="combobox"
          aria-expanded={open}
          className="inline-flex items-center justify-between gap-md min-w-[200px] max-w-[360px] pl-lg pr-md py-[6px] rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-body cursor-pointer outline-none transition-[border-color] duration-150 hover:border-border-light focus:border-accent uppercase tracking-wide"
        >
          <span className="truncate">{label}</span>
          <ChevronsUpDown size={14} className="shrink-0 text-text-muted" />
        </button>
      </PopoverTrigger>
      <PopoverContent
        className="w-[var(--radix-popover-trigger-width)] min-w-[280px] p-0 rounded-none border-border bg-bg-elevated"
        align="start"
      >
        <Command className="bg-bg-elevated rounded-none">
          <CommandInput
            placeholder="Search platforms..."
            className="font-mono text-body"
          />
          <CommandList className="max-h-[280px]">
            <CommandEmpty className="font-mono text-body text-text-muted py-xl text-center">
              No platform found.
            </CommandEmpty>
            <CommandGroup>
              <CommandItem
                value="all-platforms"
                onSelect={() => {
                  onSelect(null);
                  setOpen(false);
                }}
                className="font-mono text-body uppercase tracking-wide rounded-none cursor-pointer data-[selected=true]:bg-accent-tint-10 data-[selected=true]:text-text-primary"
              >
                All Platforms ({total})
                <Check
                  size={14}
                  className={cn(
                    "ml-auto",
                    selected === null ? "opacity-100 text-accent" : "opacity-0"
                  )}
                />
              </CommandItem>
              {platforms.map((p) => (
                <CommandItem
                  key={p.id}
                  value={p.name}
                  onSelect={() => {
                    onSelect(p.id === selected ? null : p.id);
                    setOpen(false);
                  }}
                  className="font-mono text-body uppercase tracking-wide rounded-none cursor-pointer data-[selected=true]:bg-accent-tint-10 data-[selected=true]:text-text-primary"
                >
                  <span className="truncate">
                    {p.name} ({p.rom_count})
                  </span>
                  <Check
                    size={14}
                    className={cn(
                      "ml-auto shrink-0",
                      p.id === selected
                        ? "opacity-100 text-accent"
                        : "opacity-0"
                    )}
                  />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
