import { useState, useCallback } from "react";
import type { Speaker } from "@/types";
import { SPEAKER_COLOR_CLASSES } from "@/types";
import { useSpeakerStore } from "@/stores/speakerStore";
import { commands } from "@/bindings";
import { FeatureStatusBanner } from "./FeatureStatusBanner";

function SpeakerRow({
  speaker,
  onRename,
  onToggleSelf,
}: {
  speaker: Speaker;
  onRename: (id: string, label: string) => void;
  onToggleSelf: (id: string) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(speaker.label);

  const commitRename = useCallback(() => {
    const trimmed = draft.trim();
    if (trimmed.length > 0 && trimmed !== speaker.label) {
      onRename(speaker.id, trimmed);
    } else {
      setDraft(speaker.label);
    }
    setEditing(false);
  }, [draft, speaker.id, speaker.label, onRename]);

  const startEditing = useCallback(() => {
    setDraft(speaker.label);
    setEditing(true);
  }, [speaker.label]);

  return (
    <div className="flex items-center gap-3 rounded-md border border-border px-3 py-2">
      <span
        className={`${SPEAKER_COLOR_CLASSES[speaker.color]} inline-block h-3 w-3 shrink-0 rounded-full`}
      />
      <div className="min-w-0 flex-1">
        {editing ? (
          <input
            className="w-full rounded border border-border bg-transparent px-1 text-sm outline-none focus:border-primary"
            value={draft}
            onChange={(e) => {
              setDraft(e.target.value);
            }}
            onBlur={commitRename}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitRename();
              if (e.key === "Escape") {
                setDraft(speaker.label);
                setEditing(false);
              }
            }}
            autoFocus
          />
        ) : (
          <button
            type="button"
            className="truncate text-sm font-medium hover:underline"
            onClick={startEditing}
          >
            {speaker.label}
          </button>
        )}
      </div>
      <button
        type="button"
        onClick={() => {
          onToggleSelf(speaker.id);
        }}
        className={`shrink-0 rounded px-2 py-0.5 text-xs font-medium transition-colors ${
          speaker.isSelf
            ? "bg-primary text-primary-foreground"
            : "border border-border text-muted-foreground hover:text-foreground"
        }`}
      >
        自分
      </button>
    </div>
  );
}

export function SpeakerPanel() {
  const speakers = useSpeakerStore((s) => s.speakers);
  const updateLabel = useSpeakerStore((s) => s.updateSpeakerLabel);
  const setSelf = useSpeakerStore((s) => s.setSelfSpeaker);

  const handleRename = useCallback(
    (id: string, label: string) => {
      updateLabel(id, label);
      void commands.updateSpeakerLabel(id, label).catch(() => {
        // Tauri not available
      });
    },
    [updateLabel],
  );

  const handleToggleSelf = useCallback(
    (id: string) => {
      setSelf(id);
    },
    [setSelf],
  );

  const speakerList = Array.from(speakers.values());

  if (speakerList.length === 0) {
    return (
      <div>
        <FeatureStatusBanner status="not-implemented" />
        <div className="flex items-center justify-center py-12 text-sm text-muted-foreground">
          話者はまだ検出されていません
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-3 p-4">
      {speakerList.map((speaker) => (
        <SpeakerRow
          key={speaker.id}
          speaker={speaker}
          onRename={handleRename}
          onToggleSelf={handleToggleSelf}
        />
      ))}
    </div>
  );
}
