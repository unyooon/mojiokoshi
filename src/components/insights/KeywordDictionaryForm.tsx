import type { Dispatch, SetStateAction } from "react";

export interface FormState {
  term: string;
  reading: string;
  definition: string;
  category: string;
}

const categories = ["tech_term", "proper_noun", "acronym", "custom"] as const;

interface KeywordDictionaryFormProps {
  form: FormState;
  setForm: Dispatch<SetStateAction<FormState>>;
  editingId: number | null;
  onSubmit: () => void;
  categories?: readonly string[];
}

export function KeywordDictionaryForm({
  form,
  setForm,
  editingId,
  onSubmit,
}: KeywordDictionaryFormProps) {
  return (
    <div className="px-4 py-3 border-b border-border space-y-2">
      <input
        type="text"
        placeholder="用語"
        value={form.term}
        onChange={(e) => {
          setForm({ ...form, term: e.target.value });
        }}
        className="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm text-foreground placeholder:text-muted-foreground"
      />
      <input
        type="text"
        placeholder="読み（任意）"
        value={form.reading}
        onChange={(e) => {
          setForm({ ...form, reading: e.target.value });
        }}
        className="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm text-foreground placeholder:text-muted-foreground"
      />
      <input
        type="text"
        placeholder="定義（任意）"
        value={form.definition}
        onChange={(e) => {
          setForm({ ...form, definition: e.target.value });
        }}
        className="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm text-foreground placeholder:text-muted-foreground"
      />
      <select
        value={form.category}
        onChange={(e) => {
          setForm({ ...form, category: e.target.value });
        }}
        className="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm text-foreground"
      >
        {categories.map((c) => (
          <option key={c} value={c}>
            {c}
          </option>
        ))}
      </select>
      <button
        type="button"
        onClick={onSubmit}
        disabled={!form.term.trim()}
        className="rounded-md bg-primary px-4 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {editingId !== null ? "更新" : "追加"}
      </button>
    </div>
  );
}
