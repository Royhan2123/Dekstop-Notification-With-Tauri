import { useCallback, useEffect, useState } from "react";
import { commands, type TaskDto } from "../../../shared/ipc";

/** Daftar tugas: setiap perubahan dikirim ke backend lalu daftar dimuat ulang (satu sumber kebenaran). */
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
    tasks,
    error,
    add: (title: string) => run(commands.tasksAdd(title)),
    toggle: (t: TaskDto) => run(commands.tasksSetDone(t.id, !t.done)),
    remove: (t: TaskDto) => run(commands.tasksRemove(t.id)),
    clearDone: () => run(commands.tasksClearDone()),
  };
}
