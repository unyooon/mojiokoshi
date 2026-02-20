import { describe, it, expect, vi, beforeAll, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SettingsDialog } from "../SettingsDialog";

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

describe("SettingsDialog", () => {
  const defaultProps = {
    open: false,
    onClose: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not show dialog content when closed", () => {
    render(<SettingsDialog {...defaultProps} />);
    const dialog = document.querySelector("dialog");
    expect(dialog).not.toHaveAttribute("open");
  });

  it("shows dialog with tabs when open", () => {
    render(<SettingsDialog {...defaultProps} open={true} />);
    expect(screen.getByText("Settings")).toBeVisible();
    expect(screen.getByText("General")).toBeVisible();
    expect(screen.getByText("Audio")).toBeVisible();
    expect(screen.getByText("AI")).toBeVisible();
    expect(screen.getByText("Export")).toBeVisible();
  });

  it("shows General tab content by default", () => {
    render(<SettingsDialog {...defaultProps} open={true} />);
    expect(screen.getByText("Theme")).toBeVisible();
    expect(screen.getByText("Language")).toBeVisible();
  });

  it("switches to Audio tab on click", () => {
    render(<SettingsDialog {...defaultProps} open={true} />);
    fireEvent.click(screen.getByText("Audio"));
    expect(screen.getByText("Whisper Model")).toBeVisible();
  });

  it("calls onClose when close button clicked", () => {
    const onClose = vi.fn();
    render(<SettingsDialog {...defaultProps} open={true} onClose={onClose} />);
    fireEvent.click(screen.getByText("×"));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("has m-auto class for centering", () => {
    render(<SettingsDialog {...defaultProps} open={true} />);
    const dialog = document.querySelector("dialog");
    expect(dialog?.className).toContain("m-auto");
  });

  it("calls onClose on backdrop click", () => {
    const onClose = vi.fn();
    render(<SettingsDialog {...defaultProps} open={true} onClose={onClose} />);
    const dialog = document.querySelector("dialog");
    expect(dialog).not.toBeNull();
    fireEvent.click(dialog as HTMLElement);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("closes dialog when open changes to false", () => {
    const { rerender } = render(<SettingsDialog {...defaultProps} open={true} />);
    const dialog = document.querySelector("dialog");
    expect(dialog).toHaveAttribute("open");

    rerender(<SettingsDialog {...defaultProps} open={false} />);
    expect(dialog).not.toHaveAttribute("open");
  });
});
