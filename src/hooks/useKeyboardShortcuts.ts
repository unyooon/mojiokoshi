import { useEffect } from "react";

/**
 * キーボードショートカットのハンドラー定義
 */
interface ShortcutHandlers {
  /** ⌘+Shift+R: 録音の開始/停止トグル */
  onToggleRecording: () => void;
  /** ⌘+Shift+P: 録音の一時停止/再開トグル */
  onTogglePause: () => void;
  /** ⌘+F: 検索パネルのトグル */
  onToggleSearch: () => void;
  /** ⌘+B: ブックマークの追加 */
  onAddBookmark: () => void;
  /** ⌘+E: エクスポートダイアログを開く */
  onExport: () => void;
  /** ⌘+,: 設定ダイアログを開く */
  onOpenSettings: () => void;
  /** ⌘+1: 文字起こしパネルにフォーカス */
  onFocusTranscript: () => void;
  /** ⌘+2: インサイトパネルにフォーカス */
  onFocusInsights: () => void;
  /** ⌘+I: 調査パネルを起動 */
  onInvestigate: () => void;
  /** ⌘+Shift+F: ミニビューのトグル */
  onToggleMiniView: () => void;
  /** ⌘+Shift+S: サマリーを生成 */
  onGenerateSummary: () => void;
}

/**
 * @description アプリケーション全体のキーボードショートカットを登録するフック。
 * keydown イベントで ⌘（metaKey）を使ったショートカットをハンドリングする。
 * @param handlers - 各ショートカットに対応するコールバック関数群
 * @returns void
 */
export function useKeyboardShortcuts(handlers: ShortcutHandlers): void {
  const {
    onToggleRecording,
    onTogglePause,
    onToggleSearch,
    onAddBookmark,
    onExport,
    onOpenSettings,
    onFocusTranscript,
    onFocusInsights,
    onInvestigate,
    onToggleMiniView,
    onGenerateSummary,
  } = handlers;

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!e.metaKey) return;

      if (e.shiftKey) {
        switch (e.key.toLowerCase()) {
          case "r":
            e.preventDefault();
            onToggleRecording();
            break;
          case "p":
            e.preventDefault();
            onTogglePause();
            break;
          case "f":
            e.preventDefault();
            onToggleMiniView();
            break;
          case "s":
            e.preventDefault();
            onGenerateSummary();
            break;
        }
        return;
      }

      switch (e.key.toLowerCase()) {
        case "f":
          e.preventDefault();
          onToggleSearch();
          break;
        case "b":
          e.preventDefault();
          onAddBookmark();
          break;
        case "e":
          e.preventDefault();
          onExport();
          break;
        case ",":
          e.preventDefault();
          onOpenSettings();
          break;
        case "1":
          e.preventDefault();
          onFocusTranscript();
          break;
        case "2":
          e.preventDefault();
          onFocusInsights();
          break;
        case "i":
          e.preventDefault();
          onInvestigate();
          break;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [
    onToggleRecording,
    onTogglePause,
    onToggleSearch,
    onAddBookmark,
    onExport,
    onOpenSettings,
    onFocusTranscript,
    onFocusInsights,
    onInvestigate,
    onToggleMiniView,
    onGenerateSummary,
  ]);
}
