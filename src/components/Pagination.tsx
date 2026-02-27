interface Props {
  offset: number;
  total: number;
  pageSize: number;
  onOffsetChange: (offset: number) => void;
}

export default function Pagination({ offset, total, pageSize, onOffsetChange }: Props) {
  if (total <= pageSize) return null;

  return (
    <div className="flex items-center justify-center gap-lg mt-3xl py-xl">
      <button
        className="btn btn-secondary btn-sm"
        disabled={offset === 0}
        onClick={() => onOffsetChange(Math.max(0, offset - pageSize))}
      >
        Previous
      </button>
      <span className="text-body text-text-muted">
        {offset + 1}&ndash;{Math.min(offset + pageSize, total)} of {total}
      </span>
      <button
        className="btn btn-secondary btn-sm"
        disabled={offset + pageSize >= total}
        onClick={() => onOffsetChange(offset + pageSize)}
      >
        Next
      </button>
    </div>
  );
}
