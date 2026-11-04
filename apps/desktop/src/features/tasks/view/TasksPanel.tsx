import { useEffect, useState } from "react";
import { useT } from "../../../shared/i18n";
import type { TaskDto } from "../../../shared/ipc";
import { useTasksVM } from "../viewmodel/useTasksVM";

type Tab = "tasks" | "notes";
/** Jeda sejak ketikan terakhir sampai catatan disimpan sendiri. */
const SAVE_MS = 500;

/** Baris pertama isi catatan sebagai pratinjau di daftar. */
function preview(body: string): string {
  return body.split("\n").find((l) => l.trim()) ?? "";
}

function NoteEditor({
  note,
  onSave,
  onBack,
  onConvert,
  onRemove,
}: {
  note: TaskDto;
  onSave: (title: string, body: string) => void;
  onBack: () => void;
  onConvert: () => void;
  onRemove: () => void;
}) {
  const t = useT();
  const [title, setTitle] = useState(note.title);
  const [body, setBody] = useState(note.body);
  const dirty = title !== note.title || body !== note.body;
  const valid = title.trim() !== "";

  useEffect(() => {
    if (!dirty || !valid) return;
    const id = window.setTimeout(() => onSave(title, body), SAVE_MS);
    return () => window.clearTimeout(id);
  }, [title, body, dirty, valid, onSave]);

  const flush = () => {
    if (dirty && valid) onSave(title, body);
  };

  return (
    <div className="note-editor">
      <input
        className="tasks-input"
        value={title}
        maxLength={120}
        placeholder={t("tasks.note_title")}
        onChange={(e) => setTitle(e.currentTarget.value)}
      />
      <textarea
        className="note-body"
        value={body}
        maxLength={4000}
        placeholder={t("tasks.note_body")}
        onChange={(e) => setBody(e.currentTarget.value)}
      />
      <div className="note-actions">
        <button
          type="button"
          onClick={() => {
            flush();
            onBack();
          }}
        >
          {t("tasks.back")}
        </button>
        <button
          type="button"
          onClick={() => {
            flush();
            onConvert();
          }}
        >
          {t("tasks.to_task")}
        </button>
        <button type="button" className="danger" onClick={onRemove}>
          {t("island.delete")}
        </button>
      </div>
    </div>
  );
}

/** Tugas dan catatan di pulau: tab Tugas (centang) dan Catatan (judul + isi); tugas bisa diubah jadi catatan. */
export function TasksPanel({ onClose }: { onClose: () => void }) {
  const t = useT();
  const vm = useTasksVM();
  const [tab, setTab] = useState<Tab>("tasks");
  const [draft, setDraft] = useState("");
  const [openId, setOpenId] = useState<number | null>(null);
  const hasDone = vm.todos.some((x) => x.done);
  const open = vm.notes.find((n) => n.id === openId) ?? null;

  const pick = (next: Tab) => {
    setTab(next);
    setOpenId(null);
    setDraft("");
  };

  return (
    <div className="tasks-panel" onClick={(e) => e.stopPropagation()}>
      <div className="shelf-head">
        <div className="seg" role="tablist">
          {(["tasks", "notes"] as const).map((k) => (
            <button
              key={k}
              type="button"
              role="tab"
              aria-selected={tab === k}
              className={tab === k ? "on" : ""}
              onClick={() => pick(k)}
            >
              {t(k === "tasks" ? "tasks.tab_tasks" : "tasks.tab_notes")}
              {vm[k === "tasks" ? "todos" : "notes"].length > 0 && (
                <em>{vm[k === "tasks" ? "todos" : "notes"].length}</em>
              )}
            </button>
          ))}
        </div>
        <span />
        {tab === "tasks" && hasDone && (
          <button type="button" onClick={() => void vm.clearDone()}>
            {t("tasks.clear_done")}
          </button>
        )}
        <button type="button" aria-label={t("island.dismiss")} onClick={onClose}>
          ×
        </button>
      </div>

      {open ? (
        <NoteEditor
          key={open.id}
          note={open}
          onSave={(title, body) => void vm.saveNote(open.id, title, body)}
          onBack={() => setOpenId(null)}
          onConvert={() => {
            setOpenId(null);
            void vm.convert(open);
          }}
          onRemove={() => {
            setOpenId(null);
            void vm.remove(open);
          }}
        />
      ) : (
        <>
          <input
            className="tasks-input"
            value={draft}
            placeholder={t(tab === "tasks" ? "tasks.placeholder" : "tasks.note_placeholder")}
            maxLength={120}
            onChange={(e) => setDraft(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key !== "Enter" || !draft.trim()) return;
              void vm.add(draft, tab === "notes").then(() => setDraft(""));
            }}
          />
          {vm.error && <p className="shelf-empty">{vm.error}</p>}
          {tab === "tasks" ? (
            vm.todos.length === 0 ? (
              <p className="shelf-empty">{t("tasks.empty")}</p>
            ) : (
              <ul className="tasks-rows">
                {vm.todos.map((task) => (
                  <li key={task.id} className={task.done ? "done" : ""}>
                    <label>
                      <input
                        type="checkbox"
                        checked={task.done}
                        onChange={() => void vm.toggle(task)}
                      />
                      <span title={task.title}>{task.title}</span>
                    </label>
                    <button
                      type="button"
                      className="row-act"
                      title={t("tasks.to_note")}
                      aria-label={t("tasks.to_note")}
                      onClick={() => void vm.convert(task)}
                    >
                      ✎
                    </button>
                    <button
                      type="button"
                      className="row-act"
                      aria-label={t("island.delete")}
                      onClick={() => void vm.remove(task)}
                    >
                      ×
                    </button>
                  </li>
                ))}
              </ul>
            )
          ) : vm.notes.length === 0 ? (
            <p className="shelf-empty">{t("tasks.notes_empty")}</p>
          ) : (
            <ul className="tasks-rows note-rows">
              {vm.notes.map((n) => (
                <li key={n.id}>
                  <button type="button" className="note-open" onClick={() => setOpenId(n.id)}>
                    <b>{n.title}</b>
                    <small>{preview(n.body)}</small>
                  </button>
                  <button
                    type="button"
                    className="row-act"
                    aria-label={t("island.delete")}
                    onClick={() => void vm.remove(n)}
                  >
                    ×
                  </button>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </div>
  );
}
