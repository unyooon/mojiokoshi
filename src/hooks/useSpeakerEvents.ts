import { useEffect } from "react";
import { useSpeakerStore } from "@/stores/speakerStore";

interface SpeakerDetectedPayload {
  id: string;
  label: string;
  color: string;
}

interface SpeakerUpdatedPayload {
  id: string;
  label: string;
}

export function useSpeakerEvents(isRecording: boolean) {
  const addSpeaker = useSpeakerStore((s) => s.addSpeaker);
  const updateSpeakerLabel = useSpeakerStore((s) => s.updateSpeakerLabel);
  const clearSpeakers = useSpeakerStore((s) => s.clearSpeakers);

  useEffect(() => {
    if (!isRecording) return;

    let unlistenDetected: (() => void) | null = null;
    let unlistenUpdated: (() => void) | null = null;

    const setup = async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");

        unlistenDetected = await listen<SpeakerDetectedPayload>("speaker:detected", (event) => {
          addSpeaker(event.payload.id, event.payload.label);
        });

        unlistenUpdated = await listen<SpeakerUpdatedPayload>("speaker:updated", (event) => {
          updateSpeakerLabel(event.payload.id, event.payload.label);
        });
      } catch {
        // Tauri not available (running in browser)
      }
    };

    void setup();

    return () => {
      unlistenDetected?.();
      unlistenUpdated?.();
    };
  }, [isRecording, addSpeaker, updateSpeakerLabel]);

  // Clear speakers when recording stops
  useEffect(() => {
    if (!isRecording) return;

    return () => {
      clearSpeakers();
    };
  }, [isRecording, clearSpeakers]);
}
