import { expect, it } from "vitest";
import { REMINDER_MIN, dueKinds, loadPrefs } from "./useReminders";

const allOn = { water: true, eyes: true, stretch: true };
const MIN = 60_000;

it("pengingat jatuh tempo sesuai interval masing-masing dan hanya yang menyala", () => {
  const last = { water: 0, eyes: 0, stretch: 0 };
  expect(dueKinds(10 * MIN, last, allOn)).toEqual([]);
  expect(dueKinds(REMINDER_MIN.eyes * MIN, last, allOn)).toEqual(["eyes"]);
  expect(dueKinds(REMINDER_MIN.water * MIN, last, allOn)).toEqual(["water", "eyes", "stretch"]);
  expect(dueKinds(REMINDER_MIN.water * MIN, last, { ...allOn, eyes: false })).toEqual([
    "water",
    "stretch",
  ]);
});

it("setelan rusak atau kosong berarti semua menyala, hanya false eksplisit yang mematikan", () => {
  expect(loadPrefs(null)).toEqual(allOn);
  expect(loadPrefs("{")).toEqual(allOn);
  expect(loadPrefs("[]")).toEqual(allOn);
  expect(loadPrefs(JSON.stringify({ water: false, eyes: "x" }))).toEqual({
    water: false,
    eyes: true,
    stretch: true,
  });
});
