import { useState, useEffect, useCallback } from "react";
import { commands } from "@/bindings";

type OnboardingStep = "request" | "waiting" | "complete";

interface PermissionOnboardingProps {
  /** オンボーディング完了時のコールバック */
  onComplete: () => void;
}

/**
 * @description 初回起動時に画面収録権限を案内するオンボーディングコンポーネント。
 * 権限が既に付与済みの場合は即座に onComplete を呼ぶ。
 * @param props.onComplete - 権限付与完了またはスキップ時に呼ばれるコールバック
 * @returns 権限オンボーディングUI
 */
export function PermissionOnboarding({ onComplete }: PermissionOnboardingProps) {
  const [step, setStep] = useState<OnboardingStep>("request");
  const [isChecking, setIsChecking] = useState(false);

  const checkPermission = useCallback(async () => {
    setIsChecking(true);
    try {
      const result = await commands.checkScreenCapturePermission();
      if (result.status === "ok" && result.data) {
        setStep("complete");
      }
    } catch {
      // 権限チェック失敗時は次のステップへ進む
    } finally {
      setIsChecking(false);
    }
  }, []);

  const handleOpenSystemSettings = useCallback(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("open_system_preferences_privacy");
    } catch {
      // Tauri環境外またはコマンド未実装の場合は無視
    }
    setStep("waiting");
  }, []);

  const handleNext = useCallback(async () => {
    await checkPermission();
    setStep("complete");
  }, [checkPermission]);

  const handleComplete = useCallback(() => {
    onComplete();
  }, [onComplete]);

  useEffect(() => {
    void checkPermission();
  }, [checkPermission]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-xl border border-border bg-card p-8 shadow-xl">
        {step === "request" && (
          <div className="space-y-6">
            <div className="space-y-2 text-center">
              <div className="text-4xl">🖥️</div>
              <h2 className="text-xl font-semibold">画面収録の権限が必要です</h2>
              <p className="text-sm text-muted-foreground">
                MojiOkoshi は会議の音声をキャプチャするために、macOS の画面収録権限が必要です。
              </p>
            </div>
            <div className="rounded-lg bg-muted p-4 space-y-2">
              <p className="text-sm font-medium">設定手順：</p>
              <ol className="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
                <li>「システム設定を開く」をクリック</li>
                <li>「プライバシーとセキュリティ」→「画面収録」を選択</li>
                <li>MojiOkoshi のトグルをオンにする</li>
                <li>アプリを再起動する</li>
              </ol>
            </div>
            <div className="flex flex-col gap-2">
              <button
                type="button"
                onClick={() => void handleOpenSystemSettings()}
                className="w-full rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
              >
                システム設定を開く
              </button>
              <button
                type="button"
                onClick={onComplete}
                className="w-full rounded-md border border-border px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
              >
                スキップ
              </button>
            </div>
          </div>
        )}

        {step === "waiting" && (
          <div className="space-y-6">
            <div className="space-y-2 text-center">
              <div className="text-4xl">⏳</div>
              <h2 className="text-xl font-semibold">権限を確認してください</h2>
              <p className="text-sm text-muted-foreground">
                システム設定で MojiOkoshi に画面収録の権限を付与した後、「次へ」をクリックしてください。
              </p>
            </div>
            <div className="flex flex-col gap-2">
              <button
                type="button"
                onClick={() => void handleNext()}
                disabled={isChecking}
                className="w-full rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50"
              >
                {isChecking ? "確認中..." : "次へ"}
              </button>
              <button
                type="button"
                onClick={() => {
                  setStep("request");
                }}
                className="w-full rounded-md border border-border px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
              >
                戻る
              </button>
            </div>
          </div>
        )}

        {step === "complete" && (
          <div className="space-y-6">
            <div className="space-y-2 text-center">
              <div className="text-4xl">✅</div>
              <h2 className="text-xl font-semibold">準備完了！</h2>
              <p className="text-sm text-muted-foreground">
                MojiOkoshi の設定が完了しました。録音を開始して会議の文字起こしを始めましょう。
              </p>
            </div>
            <button
              type="button"
              onClick={handleComplete}
              className="w-full rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
            >
              始める
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
