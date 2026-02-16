import { useEffect, useState } from "react";
import { healthCheck } from "./bindings";

function App() {
  const [backendStatus, setBackendStatus] = useState<string>("Connecting...");

  useEffect(() => {
    healthCheck()
      .then((status) => {
        setBackendStatus(status);
      })
      .catch((err: unknown) => {
        setBackendStatus(`Error: ${String(err)}`);
      });
  }, []);

  return (
    <main className="flex min-h-screen items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-bold">MojiOkoshi</h1>
        <p className="mt-2 text-muted-foreground">{backendStatus}</p>
      </div>
    </main>
  );
}

export default App;
