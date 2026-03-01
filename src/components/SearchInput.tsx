import { forwardRef } from "react";
import { cn } from "@/lib/utils";

interface SearchInputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type"> {
  className?: string;
}

const SearchInput = forwardRef<HTMLInputElement, SearchInputProps>(
  ({ className, ...props }, ref) => {
    return (
      <input
        ref={ref}
        type="text"
        className={cn(
          "px-lg py-[6px] rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-body placeholder:text-text-dim focus:border-accent outline-none transition-[border-color] duration-150",
          className,
        )}
        {...props}
      />
    );
  },
);

SearchInput.displayName = "SearchInput";

export default SearchInput;
