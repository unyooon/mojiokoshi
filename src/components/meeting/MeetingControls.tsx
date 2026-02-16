import { useState, useCallback } from "react";
import type { MeetingState } from "@/types";
import { Timer } from "./Timer";

interface MeetingControlsProps {
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
}

export function MeetingControls({
  onStart,
  onPause,
  onResume,
  onStop,
}: MeetingControlsProps) {
  const [meetingState, setMeetingState] = useState<MeetingState>("idle");
  const [startTime, setStartTime] = useState<number | null>(null);

  const handleStart = useCallback(() => {
    setMeetingState("recording");
    setStartTime(Date.now());
    onStart();
  }, [onStart]);

  const handlePause = useCallback(() => {
    setMeetingState("paused");
    onPause();
  }, [onPause]);

  const handleResume = useCallback(() => {
    setMeetingState("recording");
    onResume();
  }, [onResume]);

  const handleStop = useCallback(() => {
    setMeetingState("idle");
    setStartTime(null);
    onStop();
  }, [onStop]);

  const isRecording = meetingState === "recording";
  const isPaused = meetingState === "paused";
  const isIdle = meetingState === "idle";

  return (
    <div className="flex items-center gap-3 px-4 py-2 border-b border-border">
      <div className="flex items-center gap-2">
        {isRecording && (
          <span className="flex h-2 w-2">
            <span className="absolute inline-flex h-2 w-2 animate-ping rounded-full bg-red-400 opacity-75" />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-red-500" />
          </span>
        )}
        {isPaused && (
          <span className="inline-flex h-2 w-2 rounded-full bg-yellow-500" />
        )}
      </div>

      <Timer startTime={startTime} isRunning={isRecording} />

      <div className="flex items-center gap-1 ml-auto">
        {isIdle ? (
          <button
            onClick={handleStart}
            className="rounded-md bg-red-500 px-4 py-1.5 text-sm font-medium text-white hover:bg-red-600 transition-colors"
          >
            Start Recording
          </button>
        ) : (
          <>
            {isRecording ? (
              <button
                onClick={handlePause}
                className="rounded-md bg-secondary px-3 py-1.5 text-sm font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors"
              >
                Pause
              </button>
            ) : (
              <button
                onClick={handleResume}
                className="rounded-md bg-secondary px-3 py-1.5 text-sm font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors"
              >
                Resume
              </button>
            )}
            <button
              onClick={handleStop}
              className="rounded-md bg-secondary px-3 py-1.5 text-sm font-medium text-secondary-foreground hover:bg-secondary/80 transition-colors"
            >
              Stop
            </button>
          </>
        )}
      </div>
    </div>
  );
}
