import { useState, useEffect } from "react";

interface MiniState {
  speakerName: string;
  speakerColor: string;
  latestText: string;
  actionItemCount: number;
}

export function MiniView() {
  const [state, setState] = useState<MiniState>({
    speakerName: "",
    speakerColor: "",
    latestText:
      "\u6587\u5B57\u8D77\u3053\u3057\u3092\u958B\u59CB\u3057\u3066\u304F\u3060\u3055\u3044...",
    actionItemCount: 0,
  });

  useEffect(() => {
    let unlistenTranscript: (() => void) | null = null;
    let unlistenActions: (() => void) | null = null;

    const setup = async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlistenTranscript = await listen<{
          speakerName: string;
          text: string;
        }>("mini:transcript", (event) => {
          setState((prev) => ({
            ...prev,
            speakerName: event.payload.speakerName,
            latestText: event.payload.text,
          }));
        });
        unlistenActions = await listen<{ count: number }>("mini:actions", (event) => {
          setState((prev) => ({
            ...prev,
            actionItemCount: event.payload.count,
          }));
        });
      } catch {
        // Tauri not available
      }
    };
    void setup();
    return () => {
      unlistenTranscript?.();
      unlistenActions?.();
    };
  }, []);

  const handleFocusMain = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("focus_main_window");
    } catch {
      // Tauri not available
    }
  };

  return (
    <div
      onClick={() => void handleFocusMain()}
      className="h-screen w-screen cursor-pointer select-none bg-zinc-900/95 p-3 text-white"
      data-tauri-drag-region
    >
      <div className="mb-2 flex items-center justify-between">
        <div className="flex items-center gap-2">
          {state.speakerName && (
            <>
              <span className="inline-block h-2 w-2 rounded-full bg-blue-400" />
              <span className="text-xs font-medium">{state.speakerName}</span>
            </>
          )}
        </div>
        {state.actionItemCount > 0 && (
          <span className="rounded bg-primary/20 px-1.5 text-xs text-primary">
            {state.actionItemCount} \u30A2\u30AF\u30B7\u30E7\u30F3
          </span>
        )}
      </div>
      <p className="line-clamp-3 text-sm leading-relaxed opacity-90">{state.latestText}</p>
    </div>
  );
}
