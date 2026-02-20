import { describe, it, expect, vi, beforeAll, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ModelAlert } from "../ModelAlert";

beforeAll(() => {
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

describe("ModelAlert", () => {
  const defaultProps = {
    open: false,
    onOpenSettings: vi.fn(),
    onDismiss: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not show dialog content when closed", () => {
    render(<ModelAlert {...defaultProps} />);
    const dialog = document.querySelector("dialog");
    expect(dialog).not.toHaveAttribute("open");
  });

  it("shows dialog content when open", () => {
    render(<ModelAlert {...defaultProps} open={true} />);
    expect(screen.getByText("Whisper モデルが必要です")).toBeVisible();
    expect(
      screen.getByText(/音声文字起こしにはWhisperモデルのダウンロードが必要です/),
    ).toBeVisible();
  });

  it("calls onOpenSettings when settings button clicked", () => {
    const onOpenSettings = vi.fn();
    render(<ModelAlert {...defaultProps} open={true} onOpenSettings={onOpenSettings} />);
    fireEvent.click(screen.getByText("設定を開く"));
    expect(onOpenSettings).toHaveBeenCalledOnce();
  });

  it("calls onDismiss when dismiss button clicked", () => {
    const onDismiss = vi.fn();
    render(<ModelAlert {...defaultProps} open={true} onDismiss={onDismiss} />);
    fireEvent.click(screen.getByText("後で"));
    expect(onDismiss).toHaveBeenCalledOnce();
  });

  it("has m-auto class for centering", () => {
    render(<ModelAlert {...defaultProps} open={true} />);
    const dialog = document.querySelector("dialog");
    expect(dialog?.className).toContain("m-auto");
  });

  it("calls onDismiss on backdrop click", () => {
    const onDismiss = vi.fn();
    render(<ModelAlert {...defaultProps} open={true} onDismiss={onDismiss} />);
    const dialog = document.querySelector("dialog");
    expect(dialog).not.toBeNull();
    fireEvent.click(dialog as HTMLElement);
    expect(onDismiss).toHaveBeenCalledOnce();
  });

  it("closes dialog when open changes to false", () => {
    const { rerender } = render(<ModelAlert {...defaultProps} open={true} />);
    const dialog = document.querySelector("dialog");
    expect(dialog).toHaveAttribute("open");

    rerender(<ModelAlert {...defaultProps} open={false} />);
    expect(dialog).not.toHaveAttribute("open");
  });
});
