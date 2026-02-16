import { useEffect, useState } from "react";

interface TimerProps {
  startTime: number | null;
  isRunning: boolean;
}

function formatDuration(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export function Timer({ startTime, isRunning }: TimerProps) {
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (!isRunning || startTime === null) {
      return;
    }

    const interval = setInterval(() => {
      setElapsed(Date.now() - startTime);
    }, 1000);

    return () => {
      clearInterval(interval);
    };
  }, [isRunning, startTime]);

  const displayElapsed = isRunning ? elapsed : 0;

  return (
    <span className="font-mono text-sm tabular-nums">
      {formatDuration(displayElapsed)}
    </span>
  );
}
