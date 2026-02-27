import { useRef, useState, useEffect, useCallback, useMemo } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { RomWithMeta } from "../types";
import RomCard from "./RomCard";

interface Props {
  roms: RomWithMeta[];
  onSelect: (rom: RomWithMeta) => void;
  onToggleFavorite: (romId: number, favorite: boolean) => void;
}

const CARD_MIN_WIDTH = 200;
const GAP = 16;
const CARD_INFO_HEIGHT = 60; // title + platform + padding

interface VirtualGridProps {
  roms: RomWithMeta[];
  columns: number;
  rowHeight: number;
  scrollRef: React.RefObject<HTMLDivElement | null>;
  onSelect: (rom: RomWithMeta) => void;
  onToggleFavorite: (romId: number, favorite: boolean) => void;
}

function VirtualGrid({ roms, columns, rowHeight, scrollRef, onSelect, onToggleFavorite }: VirtualGridProps) {
  const rowCount = Math.ceil(roms.length / columns);

  const virtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => rowHeight,
    overscan: 3,
  });

  useEffect(() => {
    virtualizer.measure();
  }, [rowHeight, columns, virtualizer]);

  return (
    <div
      style={{
        height: virtualizer.getTotalSize(),
        width: "100%",
        position: "relative",
      }}
    >
      {virtualizer.getVirtualItems().map((virtualRow) => {
        const startIndex = virtualRow.index * columns;
        const rowRoms = roms.slice(startIndex, startIndex + columns);

        return (
          <div
            key={virtualRow.key}
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              width: "100%",
              height: virtualRow.size,
              transform: `translateY(${virtualRow.start}px)`,
            }}
          >
            <div
              className="grid gap-xl"
              style={{
                gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
              }}
            >
              {rowRoms.map((rom) => (
                <RomCard
                  key={rom.id}
                  rom={rom}
                  onClick={() => onSelect(rom)}
                  onToggleFavorite={onToggleFavorite}
                />
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export default function RomGrid({ roms, onSelect, onToggleFavorite }: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [columns, setColumns] = useState(1);
  const [rowHeight, setRowHeight] = useState(300);

  const recalcColumns = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    const width = el.clientWidth;
    const cols = Math.max(1, Math.floor((width + GAP) / (CARD_MIN_WIDTH + GAP)));
    setColumns(cols);

    const cardWidth = (width - GAP * (cols - 1)) / cols;
    const coverHeight = (cardWidth * 4) / 3; // aspect-[3/4]
    setRowHeight(Math.ceil(coverHeight + CARD_INFO_HEIGHT + GAP));
  }, []);

  useEffect(() => {
    recalcColumns();
    const el = scrollRef.current;
    if (!el) return;
    const observer = new ResizeObserver(() => recalcColumns());
    observer.observe(el);
    return () => observer.disconnect();
  }, [recalcColumns]);

  // Key that changes when the rom list changes, forcing virtualizer to fully remount
  const gridKey = useMemo(
    () => `${roms.length}-${roms[0]?.id ?? 0}-${roms[roms.length - 1]?.id ?? 0}`,
    [roms]
  );

  return (
    <div
      ref={scrollRef}
      className="overflow-y-auto"
      style={{ height: "calc(100vh - 260px)" }}
    >
      <VirtualGrid
        key={gridKey}
        roms={roms}
        columns={columns}
        rowHeight={rowHeight}
        scrollRef={scrollRef}
        onSelect={onSelect}
        onToggleFavorite={onToggleFavorite}
      />
    </div>
  );
}
