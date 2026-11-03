import { useState } from "react";
import { useT } from "../../../shared/i18n";
import { useTasksVM } from "../viewmodel/useTasksVM";

/** Daftar tugas di pulau: ketik lalu Enter untuk menambah, centang untuk menyelesaikan. */
export function TasksPanel({ onClose }: { onClose: () => void }) {
  const t = useT();
  const vm = useTasksVM();
  const [draft, setDraft] = useState("");
  const hasDone = vm.tasks.some((x) => x.done);

  return (
    <div className="tasks-panel" onClick={(e) => e.stopPropagation()}>
      <div className="shelf-head">
        <span>{t("dock.tasks")}</span>
        {hasDone && (
          <button type="button" onClick={() => void vm.clearDone()}>
            {t("tasks.clear_done")}
          </button>
        )}
        <button type="button" aria-label={t("island.dismiss")} onClick={onClose}>
          ×
        </button>
      </div>
      <input
        className="tasks-input"
        value={draft}
        placeholder={t("tasks.placeholder")}
        maxLength={120}
        onChange={(e) => setDraft(e.currentTarget.value)}
        onKeyDown={(e) => {
          if (e.key !== "Enter" || !draft.trim()) return;
          void vm.add(draft).then(() => setDraft(""));
        }}
      />
      {vm.error && <p className="shelf-empty">{vm.error}</p>}
      {vm.tasks.length === 0 ? (
        <p className="shelf-empty">{t("tasks.empty")}</p>
      ) : (
        <ul className="tasks-rows">
          {vm.tasks.map((task) => (
            <li key={task.id} className={task.done ? "done" : ""}>
              <label>
                <input type="checkbox" checked={task.done} onChange={() => void vm.toggle(task)} />
                <span title={task.title}>{task.title}</span>
              </label>
              <button
                type="button"
                aria-label={t("island.dismiss")}
                onClick={() => void vm.remove(task)}
              >
                ×
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
