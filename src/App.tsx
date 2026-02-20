import { useEffect, useState, useCallback, useMemo } from "react";
import { commands } from "./bindings";
import type { MeetingState } from "./types";
import { MeetingControls } from "./components/meeting/MeetingControls";
import { ModelAlert } from "./components/ModelAlert";
import { MainLayout } from "./components/layout/MainLayout";
import { SettingsDialog } from "./components/settings/SettingsDialog";
import { ExportDialog } from "./components/meeting/ExportDialog";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useAiAnalysis } from "./hooks/useAiAnalysis";
import { useSpeakerEvents } from "./hooks/useSpeakerEvents";
import { useTheme } from "./hooks/useTheme";
import { useModelStatus } from "./hooks/useModelStatus";
import { useSettingsStore } from "./stores/settingsStore";

function App() {
  const [backendStatus, setBackendStatus] = useState<string>("Connecting...");
  const [isConnected, setIsConnected] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [lastSessionId, setLastSessionId] = useState<string | null>(null);
  const [isRecording, setIsRecording] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [recordingStartTime, setRecordingStartTime] = useState<number | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [captureError, setCaptureError] = useState<string | null>(null);

  const theme = useSettingsStore((s) => s.settings.theme);
  const loadSettings = useSettingsStore((s) => s.loadSettings);

  useTheme(theme);
  useTauriEvents();
  useAiAnalysis(sessionId, isRecording);
  useSpeakerEvents(isRecording);

  const { modelReady, showModelAlert, dismissAlert, checkModelStatus } =
    useModelStatus(isConnected);

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  useEffect(() => {
    commands
      .healthCheck()
      .then((result) => {
        if (result.status === "ok") {
          setBackendStatus(result.data);
          setIsConnected(true);
        } else {
          setBackendStatus(`Error: ${JSON.stringify(result.error)}`);
        }
      })
      .catch((err: unknown) => {
        setBackendStatus(`Error: ${String(err)}`);
      });
  }, []);

  // Cmd+, keyboard shortcut to toggle settings
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === ",") {
        e.preventDefault();
        setSettingsOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  const meetingState: MeetingState = useMemo(() => {
    if (isRecording) return "recording";
    if (isPaused) return "paused";
    return "idle";
  }, [isRecording, isPaused]);

  const handleStart = useCallback(async () => {
    setCaptureError(null);
    const perm = await commands.checkScreenCapturePermission();
    if (perm.status === "error") {
      console.error("Permission check failed:", perm.error);
      setCaptureError("Failed to check screen capture permission");
      return;
    }
    if (!perm.data) {
      console.error("Screen capture permission denied");
      setCaptureError("Screen capture permission denied");
      return;
    }
    const result = await commands.startAudioCapture();
    if (result.status === "error") {
      console.error("Failed to start audio capture:", result.error);
      setCaptureError("Failed to start audio capture");
      return;
    }
    const id = crypto.randomUUID();
    setSessionId(id);
    setLastSessionId(id);
    setIsRecording(true);
    setIsPaused(false);
    setRecordingStartTime(Date.now());
  }, []);

  const handlePause = useCallback(async () => {
    const result = await commands.pauseAudioCapture();
    if (result.status === "ok") {
      setIsRecording(false);
      setIsPaused(true);
    } else {
      console.error("Failed to pause audio capture:", result.error);
    }
  }, []);

  const handleResume = useCallback(async () => {
    const result = await commands.resumeAudioCapture();
    if (result.status === "ok") {
      setIsRecording(true);
      setIsPaused(false);
    } else {
      console.error("Failed to resume audio capture:", result.error);
    }
  }, []);

  // Stop always resets UI state regardless of command result,
  // because the user intent to end the session should be honored.
  const handleStop = useCallback(async () => {
    const result = await commands.stopAudioCapture();
    if (result.status === "error") {
      console.error("Failed to stop audio capture:", result.error);
    }
    setIsRecording(false);
    setIsPaused(false);
    setSessionId(null);
    setRecordingStartTime(null);
  }, []);

  const handleAlertOpenSettings = useCallback(() => {
    dismissAlert();
    requestAnimationFrame(() => {
      setSettingsOpen(true);
    });
  }, [dismissAlert]);

  const handleCloseSettings = useCallback(() => {
    setSettingsOpen(false);
    checkModelStatus();
  }, [checkModelStatus]);

  const handleOpenExport = useCallback(() => {
    setExportOpen(true);
  }, []);

  const handleCloseExport = useCallback(() => {
    setExportOpen(false);
  }, []);

  if (!isConnected) {
    return (
      <main className="flex min-h-screen items-center justify-center">
        <div className="text-center">
          <h1 className="text-4xl font-bold">MojiOkoshi</h1>
          <p className="mt-2 text-muted-foreground">{backendStatus}</p>
        </div>
      </main>
    );
  }

  return (
    <div className="flex h-screen flex-col">
      <MeetingControls
        meetingState={meetingState}
        startTime={recordingStartTime}
        onStart={handleStart}
        onPause={handlePause}
        onResume={handleResume}
        onStop={handleStop}
        onExport={handleOpenExport}
        hasSession={lastSessionId !== null}
        error={captureError}
        modelReady={modelReady}
      />
      <MainLayout sessionId={lastSessionId} />
      <SettingsDialog open={settingsOpen} onClose={handleCloseSettings} />
      <ExportDialog open={exportOpen} onClose={handleCloseExport} sessionId={lastSessionId} />
      <ModelAlert
        open={showModelAlert}
        onOpenSettings={handleAlertOpenSettings}
        onDismiss={dismissAlert}
      />
    </div>
  );
}

export default App;
