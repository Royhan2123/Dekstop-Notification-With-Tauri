import { expect, it } from "vitest";
import { loadWellness } from "./useWellnessVM";

const day = "2026-11-04";
const zero = { date: day, water: 0, eyes: 0, stretch: 0, breath: 0 };

it("catatan hari ini dibaca, hari lain atau isi rusak mulai dari nol", () => {
  expect(
    loadWellness(JSON.stringify({ date: day, water: 3, eyes: 2, stretch: 1, breath: 4 }), day),
  ).toEqual({ date: day, water: 3, eyes: 2, stretch: 1, breath: 4 });
  expect(loadWellness(JSON.stringify({ date: "2026-11-03", water: 5, eyes: 5 }), day)).toEqual(
    zero,
  );
  // Catatan lama (sebelum ada peregangan/napas) tetap terbaca.
  expect(loadWellness(JSON.stringify({ date: day, water: 2, eyes: 1 }), day)).toEqual({
    ...zero,
    water: 2,
    eyes: 1,
  });
  for (const bad of [
    null,
    "",
    "{",
    "[]",
    '"x"',
    JSON.stringify({ date: day, water: "9", eyes: -4 }),
  ]) {
    expect(loadWellness(bad, day)).toEqual(zero);
  }
});

it("angka dibatasi supaya data ngawur tidak merusak tampilan", () => {
  expect(
    loadWellness(JSON.stringify({ date: day, water: 9999, eyes: 9999, stretch: 9999 }), day),
  ).toEqual({ ...zero, water: 24, eyes: 99, stretch: 99 });
});
