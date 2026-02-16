import { useState, useCallback } from "react";
import { TranscriptPanel } from "@/components/transcript/TranscriptPanel";
import { InsightsPanel } from "@/components/insights/InsightsPanel";

export function MainLayout() {
  const [splitPercent, setSplitPercent] = useState(60);
  const [isDragging, setIsDragging] = useState(false);

  const handleMouseDown = useCallback(() => {
    setIsDragging(true);
  }, []);

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!isDragging) return;
      const container = e.currentTarget;
      const rect = container.getBoundingClientRect();
      const percent = ((e.clientX - rect.left) / rect.width) * 100;
      setSplitPercent(Math.max(30, Math.min(80, percent)));
    },
    [isDragging],
  );

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  return (
    <div
      className="flex flex-1 overflow-hidden"
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    >
      <div className="overflow-hidden" style={{ width: `${splitPercent}%` }}>
        <TranscriptPanel />
      </div>
      <div
        className={`w-1 cursor-col-resize hover:bg-primary/20 transition-colors ${isDragging ? "bg-primary/30" : "bg-border"}`}
        onMouseDown={handleMouseDown}
      />
      <div className="flex-1 overflow-hidden">
        <InsightsPanel />
      </div>
    </div>
  );
}
