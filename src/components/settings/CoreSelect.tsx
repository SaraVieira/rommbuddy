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
import type { CoreInfo, EmulatorDef } from "../../types";

const EMU_PREFIX = "emu:" as const;
const RETROARCH_PREFIX = "retroarch:" as const;

export type CoreSelectValue = `emu:${string}` | `retroarch:${string}` | "";

type DecodedMapping =
  | { type: "emulator"; id: string }
  | { type: "retroarch"; coreName: string };

export function encodeMapping(emulatorType: string, coreName: string): CoreSelectValue {
  if (emulatorType !== "retroarch") return `${EMU_PREFIX}${emulatorType}`;
  return `${RETROARCH_PREFIX}${coreName}`;
}

export function decodeMapping(value: CoreSelectValue): DecodedMapping | null {
  if (!value) return null;
  if (value.startsWith(EMU_PREFIX)) return { type: "emulator", id: value.slice(EMU_PREFIX.length) };
  if (value.startsWith(RETROARCH_PREFIX)) return { type: "retroarch", coreName: value.slice(RETROARCH_PREFIX.length) };
  return null;
}

interface CoreSelectProps {
  value: CoreSelectValue;
  cores: CoreInfo[];
  emulators: EmulatorDef[];
  defaultCore?: string;
  hasRetroarchCores: boolean;
  onChange: (value: CoreSelectValue) => void;
}

export default function CoreSelect({
  value,
  cores,
  emulators,
  defaultCore,
  hasRetroarchCores,
  onChange,
}: CoreSelectProps) {
  const [open, setOpen] = useState(false);

  const getLabel = () => {
    const decoded = decodeMapping(value);
    if (!decoded) return "Select...";
    if (decoded.type === "emulator") {
      const emu = emulators.find((e) => e.id === decoded.id);
      return emu ? emu.name : decoded.id;
    }
    const core = cores.find((c) => c.core_name === decoded.coreName);
    return core ? (core.display_name || core.core_name) : decoded.coreName;
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          role="combobox"
          aria-expanded={open}
          className="inline-flex items-center justify-between gap-md w-full pl-lg pr-md py-sm rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-body cursor-pointer outline-none transition-[border-color] duration-150 hover:border-border-light focus:border-accent"
        >
          <span className="truncate">{getLabel()}</span>
          <ChevronsUpDown size={14} className="shrink-0 text-text-muted" />
        </button>
      </PopoverTrigger>
      <PopoverContent
        className="w-[var(--radix-popover-trigger-width)] min-w-[280px] p-0 rounded-none border-border bg-bg-elevated"
        align="start"
      >
        <Command className="bg-bg-elevated rounded-none">
          <CommandInput
            placeholder="Search cores..."
            className="font-mono text-body"
          />
          <CommandList className="max-h-[280px]">
            <CommandEmpty className="font-mono text-body text-text-muted py-xl text-center">
              No core found.
            </CommandEmpty>
            <CommandGroup>
              <CommandItem
                value="__none__"
                onSelect={() => {
                  onChange("" as CoreSelectValue);
                  setOpen(false);
                }}
                className="font-mono text-body rounded-none cursor-pointer data-[selected=true]:bg-accent-tint-10 data-[selected=true]:text-text-primary"
              >
                Select...
                <Check
                  size={14}
                  className={cn(
                    "ml-auto",
                    !value ? "opacity-100 text-accent" : "opacity-0",
                  )}
                />
              </CommandItem>
            </CommandGroup>
            {emulators.length > 0 && (
              <CommandGroup heading="Standalone Emulators">
                {emulators.map((emu) => {
                  const emuValue: CoreSelectValue = `emu:${emu.id}`;
                  return (
                    <CommandItem
                      key={emuValue}
                      value={`emulator ${emu.name}`}
                      onSelect={() => {
                        onChange(emuValue);
                        setOpen(false);
                      }}
                      className="font-mono text-body rounded-none cursor-pointer data-[selected=true]:bg-accent-tint-10 data-[selected=true]:text-text-primary"
                    >
                      <span className="truncate">{emu.name}</span>
                      <Check
                        size={14}
                        className={cn(
                          "ml-auto shrink-0",
                          value === emuValue
                            ? "opacity-100 text-accent"
                            : "opacity-0",
                        )}
                      />
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            )}
            {hasRetroarchCores && (
              <CommandGroup heading="RetroArch Cores">
                {cores.map((core) => {
                  const coreValue: CoreSelectValue = `retroarch:${core.core_name}`;
                  const label = core.display_name || core.core_name;
                  const isDefault = core.core_name === defaultCore;
                  return (
                    <CommandItem
                      key={coreValue}
                      value={`retroarch ${label} ${core.core_name}`}
                      onSelect={() => {
                        onChange(coreValue);
                        setOpen(false);
                      }}
                      className="font-mono text-body rounded-none cursor-pointer data-[selected=true]:bg-accent-tint-10 data-[selected=true]:text-text-primary"
                    >
                      <span className="truncate">
                        {label}
                        {isDefault ? " (recommended)" : ""}
                      </span>
                      <Check
                        size={14}
                        className={cn(
                          "ml-auto shrink-0",
                          value === coreValue
                            ? "opacity-100 text-accent"
                            : "opacity-0",
                        )}
                      />
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
