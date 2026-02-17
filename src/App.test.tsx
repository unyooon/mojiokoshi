import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import App from "./App";
import { commands } from "./bindings";

vi.mock("./bindings", () => ({
  commands: {
    healthCheck: vi.fn(),
    checkScreenCapturePermission: vi.fn(),
    startAudioCapture: vi.fn(),
    stopAudioCapture: vi.fn(),
    pauseAudioCapture: vi.fn(),
    resumeAudioCapture: vi.fn(),
  },
}));

/* eslint-disable @typescript-eslint/unbound-method */
const mockHealthCheck = vi.mocked(commands.healthCheck);
const mockCheckPermission = vi.mocked(commands.checkScreenCapturePermission);
const mockStartCapture = vi.mocked(commands.startAudioCapture);
const mockStopCapture = vi.mocked(commands.stopAudioCapture);
const mockPauseCapture = vi.mocked(commands.pauseAudioCapture);
const mockResumeCapture = vi.mocked(commands.resumeAudioCapture);
/* eslint-enable @typescript-eslint/unbound-method */

beforeAll(() => {
  // jsdom does not implement matchMedia; stub it for useTheme hook
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });

  // jsdom does not implement HTMLDialogElement methods; stub them
  if (typeof HTMLDialogElement.prototype.showModal !== "function") {
    HTMLDialogElement.prototype.showModal = function showModal(this: HTMLDialogElement) {
      this.setAttribute("open", "");
    };
  }
  if (typeof HTMLDialogElement.prototype.close !== "function") {
    HTMLDialogElement.prototype.close = function close(this: HTMLDialogElement) {
      this.removeAttribute("open");
    };
  }
});

describe("App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default: healthCheck never resolves (stays in loading state)
    mockHealthCheck.mockReturnValue(
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      new Promise(() => {}),
    );
    mockCheckPermission.mockResolvedValue({ status: "ok", data: true });
    mockStartCapture.mockResolvedValue({ status: "ok", data: null });
    mockStopCapture.mockResolvedValue({ status: "ok", data: null });
    mockPauseCapture.mockResolvedValue({ status: "ok", data: null });
    mockResumeCapture.mockResolvedValue({ status: "ok", data: null });
  });

  it("renders loading state initially", () => {
    render(<App />);

    expect(screen.getByText("MojiOkoshi")).toBeVisible();
    expect(screen.getByText("Connecting...")).toBeVisible();
  });

  it("renders MeetingControls after healthCheck success", async () => {
    mockHealthCheck.mockResolvedValue({
      status: "ok",
      data: "Backend v0.1.0 - ok",
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Start Recording")).toBeVisible();
    });
  });

  it("renders error state on healthCheck failure", async () => {
    mockHealthCheck.mockResolvedValue({
      status: "error",
      error: { Internal: "fail" },
    });

    render(<App />);

    await waitFor(() => {
      expect(
        screen.getByText((_content, element) => {
          return element?.textContent?.startsWith("Error:") ?? false;
        }),
      ).toBeVisible();
    });
  });

  it("opens Settings dialog on Cmd+, keyboard shortcut", async () => {
    mockHealthCheck.mockResolvedValue({
      status: "ok",
      data: "Backend v0.1.0 - ok",
    });

    render(<App />);

    // Wait for the connected state
    await waitFor(() => {
      expect(screen.getByText("Start Recording")).toBeVisible();
    });

    // Fire Cmd+, keyboard shortcut
    fireEvent.keyDown(window, { key: ",", metaKey: true });

    await waitFor(() => {
      expect(screen.getByText("Settings")).toBeVisible();
    });
  });

  it("shows error when screen capture permission is denied", async () => {
    mockHealthCheck.mockResolvedValue({ status: "ok", data: "ok" });
    mockCheckPermission.mockResolvedValue({ status: "ok", data: false });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Start Recording")).toBeVisible();
    });

    fireEvent.click(screen.getByText("Start Recording"));

    await waitFor(() => {
      expect(screen.getByText("Screen capture permission denied")).toBeVisible();
    });
    expect(screen.getByText("Start Recording")).toBeVisible();
  });

  it("transitions to recording state on successful start", async () => {
    mockHealthCheck.mockResolvedValue({ status: "ok", data: "ok" });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Start Recording")).toBeVisible();
    });

    fireEvent.click(screen.getByText("Start Recording"));

    await waitFor(() => {
      expect(screen.getByText("Pause")).toBeVisible();
      expect(screen.getByText("Stop")).toBeVisible();
    });
  });

  it("does not start recording when startAudioCapture fails", async () => {
    mockHealthCheck.mockResolvedValue({ status: "ok", data: "ok" });
    mockStartCapture.mockResolvedValue({
      status: "error",
      error: { AudioCapture: "device unavailable" },
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Start Recording")).toBeVisible();
    });

    fireEvent.click(screen.getByText("Start Recording"));

    await waitFor(() => {
      expect(screen.getByText("Failed to start audio capture")).toBeVisible();
    });
    expect(screen.getByText("Start Recording")).toBeVisible();
  });
});
