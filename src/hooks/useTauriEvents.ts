import { useEffect, useRef } from "react";
import type { TranscriptEntry } from "@/types";
import { useTranscriptStore } from "@/stores/transcriptStore";

type UnlistenFn = () => void;

interface TauriEventPayload<T> {
  payload: T;
}

export function useTauriEvents() {
  const addEntry = useTranscriptStore((s) => s.addEntry);
  const updatePartial = useTranscriptStore((s) => s.updatePartial);
  const unlistenersRef = useRef<UnlistenFn[]>([]);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;

    async function setupListeners() {
      try {
        const { listen } = await import("@tauri-apps/api/event");

        if (!mountedRef.current) return;

        const unlistenFinal = await listen<TranscriptEntry>(
          "transcript:final",
          (event: TauriEventPayload<TranscriptEntry>) => {
            addEntry(event.payload);
          },
        );

        const unlistenPartial = await listen<TranscriptEntry>(
          "transcript:partial",
          (event: TauriEventPayload<TranscriptEntry>) => {
            updatePartial(event.payload);
          },
        );

        // mountedRef.current may be false if cleanup ran during await
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        if (mountedRef.current) {
          unlistenersRef.current = [unlistenFinal, unlistenPartial];
        } else {
          unlistenFinal();
          unlistenPartial();
        }
      } catch {
        // Tauri API not available (running in browser)
      }
    }

    void setupListeners();

    return () => {
      mountedRef.current = false;
      for (const unlisten of unlistenersRef.current) {
        unlisten();
      }
      unlistenersRef.current = [];
    };
  }, [addEntry, updatePartial]);
}
