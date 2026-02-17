interface ExportOptions {
  include_summary: boolean;
  include_actions: boolean;
  include_keywords: boolean;
  include_transcript: boolean;
}

const CHECKBOXES: { key: keyof ExportOptions; label: string }[] = [
  { key: "include_summary", label: "サマリー" },
  { key: "include_actions", label: "アクションアイテム" },
  { key: "include_keywords", label: "キーワード" },
  { key: "include_transcript", label: "トランスクリプト" },
];

interface ExportOptionsFormProps {
  options: ExportOptions;
  onToggle: (key: keyof ExportOptions) => void;
  onExport: () => void;
  isLoading: boolean;
  disabled: boolean;
}

export type { ExportOptions };

export function ExportOptionsForm({
  options,
  onToggle,
  onExport,
  isLoading,
  disabled,
}: ExportOptionsFormProps) {
  return (
    <>
      <fieldset className="space-y-2">
        <legend className="text-sm font-medium">含める内容</legend>
        {CHECKBOXES.map((cb) => (
          <label key={cb.key} className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={options[cb.key]}
              onChange={() => {
                onToggle(cb.key);
              }}
              className="accent-primary"
            />
            {cb.label}
          </label>
        ))}
      </fieldset>
      <button
        type="button"
        onClick={onExport}
        disabled={isLoading || disabled}
        className="rounded-md bg-primary px-4 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isLoading ? "生成中..." : "Markdownエクスポート"}
      </button>
      {isLoading && (
        <div className="flex items-center gap-2">
          <div className="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent" />
          <span className="text-xs text-muted-foreground">エクスポート生成中...</span>
        </div>
      )}
    </>
  );
}
