import { expect, it } from "vitest";
import type { TaskDto } from "../../../shared/ipc";
import { splitTasks } from "./useTasksVM";

const item = (id: number, note: boolean): TaskDto => ({
  id,
  title: `t${id}`,
  done: false,
  note,
  body: "",
});

it("butir dipisah jadi tugas dan catatan tanpa mengubah urutan", () => {
  const { todos, notes } = splitTasks([
    item(1, false),
    item(2, true),
    item(3, false),
    item(4, true),
  ]);
  expect(todos.map((t) => t.id)).toEqual([1, 3]);
  expect(notes.map((t) => t.id)).toEqual([2, 4]);
  expect(splitTasks([])).toEqual({ todos: [], notes: [] });
});
