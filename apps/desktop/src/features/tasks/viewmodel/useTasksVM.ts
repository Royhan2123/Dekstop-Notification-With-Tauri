import { useCallback, useEffect, useState } from "react";
import { commands, type TaskDto } from "../../../shared/ipc";

/** Pisahkan butir menjadi tugas dan catatan; urutan dari backend dipertahankan. */
export function splitTasks(all: TaskDto[]): { todos: TaskDto[]; notes: TaskDto[] } {
  return { todos: all.filter((t) => !t.note), notes: all.filter((t) => t.note) };
}

/** Tugas dan catatan: setiap perubahan dikirim ke backend lalu daftar dimuat ulang (satu sumber kebenaran). */
export function useTasksVM() {
  const [tasks, setTasks] = useState<TaskDto[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const r = await commands.tasksList();
    if (r.status === "ok") setTasks(r.data);
    else setError(r.error);
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const run = async (op: Promise<{ status: "ok" } | { status: "error"; error: string }>) => {
    const r = await op;
    setError(r.status === "error" ? r.error : null);
    await refresh();
  };

  return {
    ...splitTasks(tasks),
    error,
    add: (title: string, note = false) => run(commands.tasksAdd(title, note)),
    toggle: (t: TaskDto) => run(commands.tasksSetDone(t.id, !t.done)),
    remove: (t: TaskDto) => run(commands.tasksRemove(t.id)),
    clearDone: () => run(commands.tasksClearDone()),
    /** Tugas ↔ catatan. */
    convert: (t: TaskDto) => run(commands.tasksSetNote(t.id, !t.note)),
    saveNote: (id: number, title: string, body: string) =>
      run(commands.tasksUpdateNote(id, title, body)),
  };
}
