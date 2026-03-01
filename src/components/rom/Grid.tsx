import { useRef, useState, useEffect, useCallback, useMemo } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { RomWithMeta } from "../../types";
import RomCard from "./Card";

interface Props {
  roms: RomWithMeta[];
  onSelect: (rom: RomWithMeta) => void;
  onToggleFavorite: (romId: number, favorite: boolean) => void;
  onLoadMore?: () => void;
  hasMore?: boolean;
  loadingMore?: boolean;
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
  loadingMore?: boolean;
  extraRows: number;
}

function VirtualGrid({
  roms,
  columns,
  rowHeight,
  scrollRef,
  onSelect,
  onToggleFavorite,
  loadingMore,
  extraRows,
}: VirtualGridProps) {
  const rowCount = Math.ceil(roms.length / columns) + extraRows;

  const virtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => rowHeight,
    overscan: 3,
  });

  useEffect(() => {
    virtualizer.measure();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rowHeight, columns]);

  const dataRowCount = Math.ceil(roms.length / columns);

  return (
    <div
      style={{
        height: virtualizer.getTotalSize(),
        width: "100%",
        position: "relative",
      }}
    >
      {virtualizer.getVirtualItems().map((virtualRow) => {
        if (virtualRow.index >= dataRowCount) {
          // Loading spinner row
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
              {loadingMore && (
                <div className="flex items-center justify-center py-xl text-text-muted text-body font-mono">
                  Loading more...
                </div>
              )}
            </div>
          );
        }

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

export default function RomGrid({
  roms,
  onSelect,
  onToggleFavorite,
  onLoadMore,
  hasMore,
  loadingMore,
}: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [columns, setColumns] = useState(1);
  const [rowHeight, setRowHeight] = useState(300);

  const recalcColumns = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    const width = el.clientWidth;
    const cols = Math.max(
      1,
      Math.floor((width + GAP) / (CARD_MIN_WIDTH + GAP)),
    );
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

  // Only remount virtualizer on filter/search reset (first rom id changes), not on appends
  const firstRomId = roms[0]?.id ?? 0;
  const gridKey = useMemo(() => `${firstRomId}`, [firstRomId]);

  const handleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el || !onLoadMore || !hasMore || loadingMore) return;
    if (el.scrollTop + el.clientHeight >= el.scrollHeight - 500) {
      onLoadMore();
    }
  }, [onLoadMore, hasMore, loadingMore]);

  const extraRows = loadingMore ? 1 : 0;

  return (
    <div
      ref={scrollRef}
      className="overflow-y-auto"
      style={{ height: "100%" }}
      onScroll={handleScroll}
    >
      <VirtualGrid
        key={gridKey}
        roms={roms}
        columns={columns}
        rowHeight={rowHeight}
        scrollRef={scrollRef}
        onSelect={onSelect}
        onToggleFavorite={onToggleFavorite}
        loadingMore={loadingMore}
        extraRows={extraRows}
      />
    </div>
  );
}
