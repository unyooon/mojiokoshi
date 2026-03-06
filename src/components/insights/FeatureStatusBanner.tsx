type FeatureStatus = "ready" | "stub" | "not-implemented";

interface FeatureStatusBannerProps {
  status: FeatureStatus;
}

const config: Record<FeatureStatus, { text: string; className: string } | null> = {
  ready: null,
  stub: {
    text: "この機能はAIサイドカー未連携のため、スタブデータを返します",
    className:
      "bg-yellow-50 border-yellow-200 text-yellow-800 dark:bg-yellow-950 dark:border-yellow-800 dark:text-yellow-200",
  },
  "not-implemented": {
    text: "この機能は未実装です",
    className:
      "bg-red-50 border-red-200 text-red-800 dark:bg-red-950 dark:border-red-800 dark:text-red-200",
  },
};

export function FeatureStatusBanner({ status }: FeatureStatusBannerProps) {
  const cfg = config[status];
  if (!cfg) return null;

  return (
    <div className={`mx-4 mt-3 px-3 py-2 text-xs rounded-md border ${cfg.className}`}>
      {cfg.text}
    </div>
  );
}

export type { FeatureStatus };
