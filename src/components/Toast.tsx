import type { ToastMessage } from "../hooks/useToast";

interface Props {
  toasts: ToastMessage[];
  onRemove: (id: number) => void;
}

const variantClasses: Record<string, string> = {
  success: "bg-accent-tint-20 text-accent border border-accent",
  error: "bg-warning-tint-20 text-error border border-error",
  info: "bg-bg-elevated text-text-primary border border-border",
};

export default function Toast({ toasts, onRemove }: Props) {
  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-xl right-xl z-[1000] flex flex-col gap-md">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`py-lg px-xl rounded-none font-mono text-body cursor-pointer max-w-[360px] animate-[toast-in_0.2s_ease-out] ${variantClasses[toast.type] ?? ""}`}
          onClick={() => onRemove(toast.id)}
        >
          {toast.message}
        </div>
      ))}
    </div>
  );
}
