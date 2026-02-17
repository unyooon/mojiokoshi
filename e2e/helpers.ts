import type { Page } from "@playwright/test";

/**
 * Inject a `__TAURI_INTERNALS__` mock into the page before it loads,
 * allowing the React app to run in a plain browser without the real
 * Tauri backend.
 *
 * The mock handles the `health_check` and `get_all_settings` commands
 * that the app issues on startup, as well as event-system primitives
 * (`transformCallback`, `plugin:event|listen`, etc.) so that hooks
 * relying on `@tauri-apps/api/event` do not throw.
 */
export async function mockTauriBackend(page: Page): Promise<void> {
  await page.addInitScript(() => {
    // ---- callback registry used by transformCallback / event listeners ----
    const callbacks = new Map<number, (...args: unknown[]) => void>();
    let nextId = 1;

    // ---- event listener bookkeeping (so listen returns a valid id) ----
    let nextEventId = 1;

    (window as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        // health_check: return a plain string (Result wrapper is added by bindings.ts)
        if (cmd === "health_check") {
          return Promise.resolve("Backend v0.1.0 - ok");
        }

        // settings store calls invoke("get_all_settings") directly
        if (cmd === "get_all_settings") {
          return Promise.resolve({
            theme: "system",
            language: "ja",
            fontSize: "14",
            whisperModel: "large-v3-turbo",
            vadSensitivity: "0.5",
            analysisIntervalMinutes: "3",
          });
        }

        // set_setting: fire-and-forget, just resolve
        if (cmd === "set_setting") {
          return Promise.resolve(null);
        }

        // get_setting: return null (not found)
        if (cmd === "get_setting") {
          return Promise.resolve(null);
        }

        // event plugin: register listener - return a numeric event id
        if (cmd === "plugin:event|listen") {
          const id = nextEventId++;
          return Promise.resolve(id);
        }

        // event plugin: unlisten
        if (cmd === "plugin:event|unlisten") {
          return Promise.resolve(null);
        }

        // Tauri version plugin
        if (cmd === "plugin:tauri|version") {
          return Promise.resolve("2.0.0");
        }

        // Default: resolve null for any unknown command to avoid hard failures
        if (typeof args !== "undefined") {
          // eslint-disable-next-line no-console
          console.debug(`[tauri-mock] unhandled invoke: ${cmd}`, args);
        }
        return Promise.resolve(null);
      },

      transformCallback: (cb?: (...args: unknown[]) => void, _once?: boolean) => {
        const id = nextId++;
        if (cb) {
          callbacks.set(id, cb);
        }
        return id;
      },

      unregisterCallback: (id: number) => {
        callbacks.delete(id);
      },

      convertFileSrc: (src: string) => src,

      metadata: {
        currentWebview: { label: "main" },
        currentWindow: { label: "main" },
      },
    };

    // The event plugin also accesses this separate global
    (window as Record<string, unknown>).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
      unregisterListener: () => {
        /* no-op */
      },
    };
  });
}
