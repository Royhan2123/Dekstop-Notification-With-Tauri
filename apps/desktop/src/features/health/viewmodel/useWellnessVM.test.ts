import { expect, it } from "vitest";
import { loadWellness } from "./useWellnessVM";

const day = "2026-11-04";

it("catatan hari ini dibaca, hari lain atau isi rusak mulai dari nol", () => {
  expect(loadWellness(JSON.stringify({ date: day, water: 3, eyes: 2 }), day)).toEqual({
    date: day,
    water: 3,
    eyes: 2,
  });
  expect(loadWellness(JSON.stringify({ date: "2026-11-03", water: 5, eyes: 5 }), day)).toEqual({
    date: day,
    water: 0,
    eyes: 0,
  });
  for (const bad of [
    null,
    "",
    "{",
    "[]",
    '"x"',
    JSON.stringify({ date: day, water: "9", eyes: -4 }),
  ]) {
    expect(loadWellness(bad, day)).toEqual({ date: day, water: 0, eyes: 0 });
  }
});

it("angka dibatasi supaya data ngawur tidak merusak tampilan", () => {
  expect(loadWellness(JSON.stringify({ date: day, water: 9999, eyes: 9999 }), day)).toEqual({
    date: day,
    water: 24,
    eyes: 99,
  });
});
