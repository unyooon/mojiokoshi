import { useEffect } from "react";
import { useMeetingLinkStore } from "@/stores/meetingLinkStore";

interface MeetingLinksPanelProps {
  sessionId?: string | null;
}

export function MeetingLinksPanel({ sessionId = null }: MeetingLinksPanelProps) {
  const links = useMeetingLinkStore((s) => s.links);
  const isSearching = useMeetingLinkStore((s) => s.isSearching);
  const findRelated = useMeetingLinkStore((s) => s.findRelated);
  const loadLinks = useMeetingLinkStore((s) => s.loadLinks);

  useEffect(() => {
    if (sessionId) {
      void loadLinks(sessionId);
    }
  }, [sessionId, loadLinks]);

  if (isSearching) {
    return (
      <div className="flex flex-col items-center justify-center py-12 gap-3">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        <span className="text-sm text-muted-foreground">関連会議を検索中...</span>
      </div>
    );
  }

  if (links.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12 gap-4">
        <p className="text-sm text-muted-foreground">関連する過去の会議はありません</p>
        <button
          type="button"
          onClick={() => {
            if (sessionId) void findRelated(sessionId);
          }}
          disabled={!sessionId}
          className="rounded-md bg-primary px-4 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          関連会議を検索
        </button>
      </div>
    );
  }

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">関連会議</h3>
        <button
          type="button"
          onClick={() => {
            if (sessionId) void findRelated(sessionId);
          }}
          disabled={!sessionId}
          className="rounded-md bg-secondary px-3 py-1 text-xs font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          再検索
        </button>
      </div>
      <div className="space-y-3">
        {links.map((link) => (
          <div key={link.id} className="rounded-lg border border-border bg-card p-3 space-y-2">
            <div className="flex items-start justify-between gap-2">
              <h4 className="text-sm font-medium leading-snug">{link.related_title}</h4>
              <span className="shrink-0 rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
                {Math.round(link.similarity_score * 100)}% 一致
              </span>
            </div>
            {link.shared_keywords.length > 0 && (
              <div className="flex flex-wrap gap-1">
                {link.shared_keywords.map((keyword) => (
                  <span
                    key={keyword}
                    className="inline-block rounded-full bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground"
                  >
                    {keyword}
                  </span>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
