import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { RomWithMeta } from "../../types";
import RomRow from "./Row";

interface Props {
  roms: RomWithMeta[];
  onSelect: (rom: RomWithMeta) => void;
  onToggleFavorite: (romId: number, favorite: boolean) => void;
}

const ROW_HEIGHT = 53; // ~42px image + py-1.5 (12px) ≈ 53px

export default function RomList({ roms, onSelect, onToggleFavorite }: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: roms.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 10,
  });

  return (
    <div
      ref={scrollRef}
      className="overflow-y-auto"
      style={{ height: "calc(100vh - 260px)" }}
    >
      <table className="w-full border-collapse">
        <thead>
          <tr className="[&>th]:text-left [&>th]:py-md [&>th]:px-lg [&>th]:text-nav [&>th]:font-medium [&>th]:text-text-muted [&>th]:uppercase [&>th]:tracking-wide [&>th]:border-b [&>th]:border-border">
            <th style={{ width: 32 }}></th>
            <th style={{ width: 40 }}></th>
            <th>Name</th>
            <th>Platform</th>
            <th>Region</th>
            <th>Size</th>
            <th style={{ width: 28 }}></th>
          </tr>
        </thead>
        <tbody>
          {virtualizer.getVirtualItems().length > 0 && (
            <tr>
              <td
                colSpan={7}
                style={{
                  height: virtualizer.getVirtualItems()[0].start,
                  padding: 0,
                  border: "none",
                }}
              />
            </tr>
          )}
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const rom = roms[virtualRow.index];
            return (
              <RomRow
                key={rom.id}
                rom={rom}
                onClick={() => onSelect(rom)}
                onToggleFavorite={onToggleFavorite}
              />
            );
          })}
          {virtualizer.getVirtualItems().length > 0 && (
            <tr>
              <td
                colSpan={7}
                style={{
                  height:
                    virtualizer.getTotalSize() -
                    (virtualizer.getVirtualItems()[
                      virtualizer.getVirtualItems().length - 1
                    ]?.end ?? 0),
                  padding: 0,
                  border: "none",
                }}
              />
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
