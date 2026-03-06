import { useState, useCallback, useEffect, type ReactNode } from "react";

/**
 * TranscriptContextMenu のプロパティ
 */
interface TranscriptContextMenuProps {
  /** ラップ対象の子要素 */
  children: ReactNode;
  /** 調査を実行するコールバック */
  onInvestigate: () => void;
  /** 現在調査中かどうか */
  isInvestigating: boolean;
}

/**
 * @description
 * トランスクリプト上のテキスト選択に対してコンテキストメニューと
 * ⌘+I キーボードショートカットによる Claude 調査機能を提供するコンポーネント。
 *
 * @param props - コンポーネントのプロパティ
 * @param props.children - ラップ対象の子要素
 * @param props.onInvestigate - 調査を実行するコールバック
 * @param props.isInvestigating - 現在調査中かどうか
 * @returns コンテキストメニューでラップされた子要素
 */
export function TranscriptContextMenu({
  children,
  onInvestigate,
  isInvestigating,
}: TranscriptContextMenuProps) {
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);

  const onCtxMenu = useCallback((e: React.MouseEvent) => {
    const sel = window.getSelection();
    if (sel && !sel.isCollapsed) {
      e.preventDefault();
      setMenu({ x: e.clientX, y: e.clientY });
    }
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === "i") {
        e.preventDefault();
        onInvestigate();
      }
    };
    const onClick = () => {
      setMenu(null);
    };
    document.addEventListener("keydown", onKey);
    document.addEventListener("click", onClick);
    return () => {
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("click", onClick);
    };
  }, [onInvestigate]);

  const handleInvestigateClick = useCallback(() => {
    setMenu(null);
    onInvestigate();
  }, [onInvestigate]);

  return (
    <div onContextMenu={onCtxMenu} className="contents">
      {children}
      {menu && (
        <div
          className="fixed z-50 min-w-[180px] py-1 rounded-md border border-border bg-popover text-popover-foreground shadow-md"
          style={{ left: menu.x, top: menu.y }}
        >
          <button
            type="button"
            onClick={handleInvestigateClick}
            disabled={isInvestigating}
            className="w-full px-3 py-1.5 text-left text-sm hover:bg-accent transition-colors disabled:opacity-50"
          >
            {isInvestigating ? "調査中..." : "Claudeで調査させる"}
          </button>
        </div>
      )}
    </div>
  );
}
