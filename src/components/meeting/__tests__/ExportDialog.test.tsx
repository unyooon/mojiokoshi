import { describe, it, expect, vi, beforeAll, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ExportDialog } from "../ExportDialog";

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

describe("ExportDialog", () => {
  const defaultProps = {
    open: false,
    onClose: vi.fn(),
    sessionId: "test-session-123",
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not show dialog content when closed", () => {
    render(<ExportDialog {...defaultProps} />);
    const dialog = document.querySelector("dialog");
    expect(dialog).not.toHaveAttribute("open");
  });

  it("shows dialog with export form when open", () => {
    render(<ExportDialog {...defaultProps} open={true} />);
    expect(screen.getByText("エクスポート")).toBeVisible();
    expect(screen.getByText("含める内容")).toBeVisible();
    expect(screen.getByText("Markdownエクスポート")).toBeVisible();
  });

  it("calls onClose when close button clicked", () => {
    const onClose = vi.fn();
    render(<ExportDialog {...defaultProps} open={true} onClose={onClose} />);
    fireEvent.click(screen.getByText("×"));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("has m-auto class for centering", () => {
    render(<ExportDialog {...defaultProps} open={true} />);
    const dialog = document.querySelector("dialog");
    expect(dialog?.className).toContain("m-auto");
  });

  it("calls onClose on backdrop click", () => {
    const onClose = vi.fn();
    render(<ExportDialog {...defaultProps} open={true} onClose={onClose} />);
    const dialog = document.querySelector("dialog");
    expect(dialog).not.toBeNull();
    fireEvent.click(dialog as HTMLElement);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("closes dialog when open changes to false", () => {
    const { rerender } = render(<ExportDialog {...defaultProps} open={true} />);
    const dialog = document.querySelector("dialog");
    expect(dialog).toHaveAttribute("open");

    rerender(<ExportDialog {...defaultProps} open={false} />);
    expect(dialog).not.toHaveAttribute("open");
  });

  it("disables export button when sessionId is null", () => {
    render(<ExportDialog {...defaultProps} open={true} sessionId={null} />);
    const btn = screen.getByText("Markdownエクスポート");
    expect(btn).toBeDisabled();
  });
});
