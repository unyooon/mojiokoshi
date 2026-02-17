import { useEffect } from "react";

export type Theme = "light" | "dark" | "system";

/**
 * React hook that toggles the `dark` class on `<html>` based on the
 * chosen theme. When theme is "system", it listens for OS preference
 * changes via `matchMedia`.
 */
export function useTheme(theme: Theme) {
  useEffect(() => {
    const root = document.documentElement;

    if (theme === "system") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      const apply = () => {
        root.classList.toggle("dark", mq.matches);
      };
      apply();
      mq.addEventListener("change", apply);
      return () => {
        mq.removeEventListener("change", apply);
      };
    }

    root.classList.toggle("dark", theme === "dark");
  }, [theme]);
}

/**
 * Imperative helper for entry-point scripts that are not React
 * components (e.g. mini.tsx). Applies "system" theme and listens
 * for OS preference changes.
 */
export function applySystemTheme(): () => void {
  const root = document.documentElement;
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const apply = () => {
    root.classList.toggle("dark", mq.matches);
  };
  apply();
  mq.addEventListener("change", apply);
  return () => {
    mq.removeEventListener("change", apply);
  };
}
