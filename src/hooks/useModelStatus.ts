import { useState, useCallback, useEffect } from "react";
import { getWhisperModelStatus, type WhisperModelStatus } from "@/commands/whisperModel";

export function useModelStatus(isConnected: boolean) {
  const [modelReady, setModelReady] = useState(false);
  const [showModelAlert, setShowModelAlert] = useState(false);

  const checkModelStatus = useCallback(() => {
    getWhisperModelStatus()
      .then((result) => {
        if (result.status === "ok") {
          const data: WhisperModelStatus = result.data;
          if (data === "NotDownloaded") {
            setModelReady(false);
            setShowModelAlert(true);
          } else if (typeof data === "object" && "Ready" in data) {
            setModelReady(true);
          }
        }
      })
      .catch((err: unknown) => {
        console.error("Failed to check model status:", err);
      });
  }, []);

  useEffect(() => {
    if (!isConnected) return;
    checkModelStatus();
  }, [isConnected, checkModelStatus]);

  const dismissAlert = useCallback(() => {
    setShowModelAlert(false);
  }, []);

  return { modelReady, showModelAlert, dismissAlert, checkModelStatus };
}
