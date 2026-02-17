import { useCallback, useEffect, useState } from "react";
import type { DictionaryKeyword } from "@/types";
import { useKeywordDictionaryStore } from "@/stores/keywordDictionaryStore";
import { KeywordDictionaryForm, type FormState } from "./KeywordDictionaryForm";

const catColors: Record<string, string> = {
  tech_term: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  proper_noun: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
  acronym: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  custom: "bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200",
};

const emptyForm: FormState = { term: "", reading: "", definition: "", category: "custom" };

export function KeywordDictionaryPanel() {
  const { keywords, isLoading, loadKeywords, addKeyword, updateKeyword, deleteKeyword } =
    useKeywordDictionaryStore();

  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [form, setForm] = useState<FormState>(emptyForm);

  useEffect(() => {
    void loadKeywords();
  }, [loadKeywords]);

  const resetForm = useCallback(() => {
    setForm(emptyForm);
    setShowForm(false);
    setEditingId(null);
  }, []);

  const handleSubmit = useCallback(() => {
    if (!form.term.trim()) return;
    const reading = form.reading.trim() || null;
    const definition = form.definition.trim() || null;
    if (editingId !== null) {
      void updateKeyword(editingId, form.term.trim(), reading, definition, form.category);
    } else {
      void addKeyword(form.term.trim(), reading, definition, form.category);
    }
    resetForm();
  }, [form, editingId, addKeyword, updateKeyword, resetForm]);

  const handleEdit = useCallback((kw: DictionaryKeyword) => {
    setEditingId(kw.id);
    setForm({
      term: kw.term,
      reading: kw.reading ?? "",
      definition: kw.definition ?? "",
      category: kw.category,
    });
    setShowForm(true);
  }, []);

  const handleDelete = useCallback(
    (id: number) => void (window.confirm("このキーワードを削除しますか？") && deleteKeyword(id)),
    [deleteKeyword],
  );

  if (isLoading) {
    return (
      <div className="flex flex-col items-center justify-center py-12 gap-3">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        <span className="text-sm text-muted-foreground">読み込み中...</span>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <span className="text-sm font-medium text-foreground">カスタム辞書</span>
        <button
          type="button"
          onClick={() => {
            setShowForm(!showForm);
            setEditingId(null);
            setForm(emptyForm);
          }}
          className="rounded-md bg-primary px-3 py-1 text-xs font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          {showForm ? "キャンセル" : "キーワード追加"}
        </button>
      </div>

      {showForm && (
        <KeywordDictionaryForm
          form={form}
          setForm={setForm}
          editingId={editingId}
          onSubmit={handleSubmit}
        />
      )}

      <div className="flex-1 overflow-y-auto">
        {keywords.length === 0 ? (
          <div className="flex items-center justify-center py-12 text-muted-foreground text-sm">
            辞書にキーワードがありません
          </div>
        ) : (
          <ul className="divide-y divide-border">
            {keywords.map((kw) => (
              <li key={kw.id} className="px-4 py-3 flex items-start justify-between gap-2">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-foreground truncate">{kw.term}</span>
                    {kw.reading && (
                      <span className="text-xs text-muted-foreground">({kw.reading})</span>
                    )}
                    <span
                      className={`inline-block rounded-full px-2 py-0.5 text-[10px] font-medium ${catColors[kw.category] ?? catColors.custom}`}
                    >
                      {kw.category}
                    </span>
                  </div>
                  {kw.definition && (
                    <p className="text-xs text-muted-foreground mt-1 truncate">{kw.definition}</p>
                  )}
                </div>
                <div className="flex gap-1 shrink-0">
                  <button
                    type="button"
                    onClick={() => {
                      handleEdit(kw);
                    }}
                    className="rounded px-2 py-1 text-xs text-muted-foreground hover:bg-secondary transition-colors"
                  >
                    編集
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      handleDelete(kw.id);
                    }}
                    className="rounded px-2 py-1 text-xs text-destructive hover:bg-destructive/10 transition-colors"
                  >
                    削除
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
