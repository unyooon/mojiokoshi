import { useEffect, useRef } from "react";
import type { TranscriptEntry } from "@/types";
import { useTranscriptStore } from "@/stores/transcriptStore";

type UnlistenFn = () => void;

interface TauriEventPayload<T> {
  payload: T;
}

interface TranscriptEvent {
  id: string;
  timestamp: number;
  text: string;
  startMs: number;
  endMs: number;
  confidence: number;
  isPartial: boolean;
}

function toTranscriptEntry(event: TranscriptEvent): TranscriptEntry {
  return {
    id: event.id,
    timestamp: event.timestamp,
    text: event.text,
    confidence: event.confidence,
    isPartial: event.isPartial,
  };
}

export function useTauriEvents() {
  const addEntry = useTranscriptStore((s) => s.addEntry);
  const updatePartial = useTranscriptStore((s) => s.updatePartial);
  const unlistenersRef = useRef<UnlistenFn[]>([]);

  useEffect(() => {
    // Local variable per effect invocation — immune to StrictMode race
    let mounted = true;

    async function setupListeners() {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        if (!mounted) return;

        const unlistenFinal = await listen<TranscriptEvent>(
          "transcript:final",
          (event: TauriEventPayload<TranscriptEvent>) => {
            addEntry(toTranscriptEntry(event.payload));
          },
        );
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- async race with StrictMode cleanup
        if (!mounted) {
          unlistenFinal();
          return;
        }

        const unlistenPartial = await listen<TranscriptEvent>(
          "transcript:partial",
          (event: TauriEventPayload<TranscriptEvent>) => {
            updatePartial(toTranscriptEntry(event.payload));
          },
        );
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- async race with StrictMode cleanup
        if (!mounted) {
          unlistenFinal();
          unlistenPartial();
          return;
        }

        unlistenersRef.current = [unlistenFinal, unlistenPartial];
      } catch {
        // Tauri API not available (running in browser)
      }
    }

    void setupListeners();

    return () => {
      mounted = false;
      for (const unlisten of unlistenersRef.current) {
        unlisten();
      }
      unlistenersRef.current = [];
    };
  }, [addEntry, updatePartial]);
}
