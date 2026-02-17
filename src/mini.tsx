import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import { MiniView } from "./components/mini/MiniView";

const rootElement = document.getElementById("root");
if (rootElement) {
  createRoot(rootElement).render(
    <StrictMode>
      <MiniView />
    </StrictMode>,
  );
}
