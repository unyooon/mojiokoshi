import { useRef, useEffect, useCallback } from "react";
import { useSearchStore } from "@/stores/searchStore";

export function SearchBar() {
  const inputRef = useRef<HTMLInputElement>(null);
  const query = useSearchStore((s) => s.query);
  const matches = useSearchStore((s) => s.matches);
  const currentMatchIndex = useSearchStore((s) => s.currentMatchIndex);
  const setQuery = useSearchStore((s) => s.setQuery);
  const close = useSearchStore((s) => s.close);
  const nextMatch = useSearchStore((s) => s.nextMatch);
  const prevMatch = useSearchStore((s) => s.prevMatch);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        close();
        return;
      }

      const isNext =
        (e.key === "Enter" && !e.shiftKey) || (e.key === "g" && e.metaKey && !e.shiftKey);
      const isPrev =
        (e.key === "Enter" && e.shiftKey) || (e.key === "g" && e.metaKey && e.shiftKey);

      if (isNext) {
        e.preventDefault();
        nextMatch();
      } else if (isPrev) {
        e.preventDefault();
        prevMatch();
      }
    },
    [close, nextMatch, prevMatch],
  );

  const matchDisplay =
    matches.length > 0 ? `${currentMatchIndex + 1}/${matches.length} 件` : query ? "0 件" : "";

  return (
    <div className="absolute top-0 left-0 right-0 z-10 flex items-center gap-2 px-3 py-2 bg-background/95 backdrop-blur border-b border-border shadow-sm">
      <svg
        className="shrink-0 w-4 h-4 text-muted-foreground"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
        strokeWidth={2}
      >
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.35-4.35" />
      </svg>
      <input
        ref={inputRef}
        type="text"
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
        }}
        onKeyDown={handleKeyDown}
        placeholder="検索..."
        className="flex-1 min-w-0 bg-transparent text-sm text-foreground placeholder:text-muted-foreground outline-none"
        aria-label="Search transcript"
      />
      {matchDisplay && (
        <span className="shrink-0 text-xs text-muted-foreground tabular-nums">{matchDisplay}</span>
      )}
      <button
        type="button"
        onClick={prevMatch}
        disabled={matches.length === 0}
        className="shrink-0 p-1 rounded hover:bg-muted disabled:opacity-30 transition-colors"
        aria-label="Previous match"
      >
        <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m18 15-6-6-6 6" />
        </svg>
      </button>
      <button
        type="button"
        onClick={nextMatch}
        disabled={matches.length === 0}
        className="shrink-0 p-1 rounded hover:bg-muted disabled:opacity-30 transition-colors"
        aria-label="Next match"
      >
        <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m6 9 6 6 6-6" />
        </svg>
      </button>
      <button
        type="button"
        onClick={close}
        className="shrink-0 p-1 rounded hover:bg-muted transition-colors"
        aria-label="Close search"
      >
        <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M6 18 18 6M6 6l12 12"
          />
        </svg>
      </button>
    </div>
  );
}
