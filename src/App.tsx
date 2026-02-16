import { useEffect, useState, useCallback } from "react";
import { healthCheck } from "./bindings";
import { MeetingControls } from "./components/meeting/MeetingControls";
import { MainLayout } from "./components/layout/MainLayout";
import { useTauriEvents } from "./hooks/useTauriEvents";

function App() {
  const [backendStatus, setBackendStatus] = useState<string>("Connecting...");
  const [isConnected, setIsConnected] = useState(false);

  useTauriEvents();

  useEffect(() => {
    healthCheck()
      .then((status) => {
        setBackendStatus(status);
        setIsConnected(true);
      })
      .catch((err: unknown) => {
        setBackendStatus(`Error: ${String(err)}`);
      });
  }, []);

  const handleStart = useCallback(() => {
    // Will invoke Tauri command
  }, []);

  const handlePause = useCallback(() => {
    // Will invoke Tauri command
  }, []);

  const handleResume = useCallback(() => {
    // Will invoke Tauri command
  }, []);

  const handleStop = useCallback(() => {
    // Will invoke Tauri command
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
        onStart={handleStart}
        onPause={handlePause}
        onResume={handleResume}
        onStop={handleStop}
      />
      <MainLayout />
    </div>
  );
}

export default App;
