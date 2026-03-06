import { useEffect, useState, useCallback, useMemo } from "react";
import { commands } from "./bindings";
import type { MeetingState } from "./types";
import { MeetingControls } from "./components/meeting/MeetingControls";
import { ModelAlert } from "./components/ModelAlert";
import { MainLayout } from "./components/layout/MainLayout";
import { SettingsDialog } from "./components/settings/SettingsDialog";
import { ExportDialog } from "./components/meeting/ExportDialog";
import { PermissionOnboarding } from "./components/onboarding/PermissionOnboarding";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useAiAnalysis } from "./hooks/useAiAnalysis";
import { useSpeakerEvents } from "./hooks/useSpeakerEvents";
import { useTheme } from "./hooks/useTheme";
import { useModelStatus } from "./hooks/useModelStatus";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
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
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [permissionChecked, setPermissionChecked] = useState(false);

  const theme = useSettingsStore((s) => s.settings.theme);
  const whisperModel = useSettingsStore((s) => s.settings.whisperModel);
  const loadSettings = useSettingsStore((s) => s.loadSettings);

  useTheme(theme);
  useTauriEvents();
  useAiAnalysis(sessionId, isRecording);
  useSpeakerEvents(isRecording);

  const { modelReady, showModelAlert, dismissAlert, checkModelStatus } = useModelStatus(
    isConnected,
    whisperModel,
  );

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  // 初回起動時の権限チェック
  useEffect(() => {
    async function checkPermission() {
      try {
        const result = await commands.checkScreenCapturePermission();
        if (result.status === "ok" && !result.data) {
          setShowOnboarding(true);
        }
      } catch {
        // 権限チェック失敗時はオンボーディングをスキップ
      } finally {
        setPermissionChecked(true);
      }
    }
    void checkPermission();
  }, []);

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

  const meetingState: MeetingState = useMemo(() => {
    if (isRecording) return "recording";
    if (isPaused) return "paused";
    return "idle";
  }, [isRecording, isPaused]);

  const handleStart = useCallback(async () => {
    setCaptureError(null);
    const perm = await commands.checkScreenCapturePermission();
    if (perm.status === "error") {
      setCaptureError("Failed to check screen capture permission");
      return;
    }
    if (!perm.data) {
      setCaptureError("Screen capture permission denied");
      return;
    }
    const result = await commands.startAudioCapture();
    if (result.status === "error") {
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
    }
  }, []);

  const handleResume = useCallback(async () => {
    const result = await commands.resumeAudioCapture();
    if (result.status === "ok") {
      setIsRecording(true);
      setIsPaused(false);
    }
  }, []);

  // Stop always resets UI state regardless of command result,
  // because the user intent to end the session should be honored.
  const handleStop = useCallback(async () => {
    await commands.stopAudioCapture();
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
    checkModelStatus(whisperModel);
  }, [checkModelStatus, whisperModel]);

  const handleOpenExport = useCallback(() => {
    setExportOpen(true);
  }, []);

  const handleCloseExport = useCallback(() => {
    setExportOpen(false);
  }, []);

  const handleOpenSettings = useCallback(() => {
    setSettingsOpen((prev) => !prev);
  }, []);

  const handleToggleRecording = useCallback(() => {
    if (isRecording || isPaused) {
      void handleStop();
    } else {
      void handleStart();
    }
  }, [isRecording, isPaused, handleStart, handleStop]);

  const handleTogglePause = useCallback(() => {
    if (isRecording) {
      void handlePause();
    } else if (isPaused) {
      void handleResume();
    }
  }, [isRecording, isPaused, handlePause, handleResume]);

  const handleToggleSearch = useCallback(() => {
    // 検索パネルのトグル（将来的な実装のためのプレースホルダー）
  }, []);

  const handleAddBookmark = useCallback(() => {
    // ブックマーク追加（将来的な実装のためのプレースホルダー）
  }, []);

  const handleFocusTranscript = useCallback(() => {
    // 文字起こしパネルフォーカス（将来的な実装のためのプレースホルダー）
  }, []);

  const handleFocusInsights = useCallback(() => {
    // インサイトパネルフォーカス（将来的な実装のためのプレースホルダー）
  }, []);

  const handleInvestigate = useCallback(() => {
    // 調査パネル起動（将来的な実装のためのプレースホルダー）
  }, []);

  const handleToggleMiniView = useCallback(() => {
    void commands.toggleMiniView();
  }, []);

  const handleGenerateSummary = useCallback(() => {
    if (!sessionId) return;
    void commands.generateMinutes(sessionId);
  }, [sessionId]);

  const handleOnboardingComplete = useCallback(() => {
    setShowOnboarding(false);
  }, []);

  useKeyboardShortcuts({
    onToggleRecording: handleToggleRecording,
    onTogglePause: handleTogglePause,
    onToggleSearch: handleToggleSearch,
    onAddBookmark: handleAddBookmark,
    onExport: handleOpenExport,
    onOpenSettings: handleOpenSettings,
    onFocusTranscript: handleFocusTranscript,
    onFocusInsights: handleFocusInsights,
    onInvestigate: handleInvestigate,
    onToggleMiniView: handleToggleMiniView,
    onGenerateSummary: handleGenerateSummary,
  });

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
      {permissionChecked && showOnboarding && (
        <PermissionOnboarding onComplete={handleOnboardingComplete} />
      )}
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
