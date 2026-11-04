import { expect, it } from "vitest";
import {
  BREATH_CYCLES,
  PATTERNS,
  SMALL,
  breathReducer,
  scaleAfter,
  type BreathState,
} from "./useBreathVM";

const idle: BreathState = { running: false, step: 0, cycle: 0, done: false };

it("sesi napas berjalan langkah demi langkah, siklus demi siklus, lalu selesai", () => {
  const n = PATTERNS.box.length;
  let s = breathReducer(idle, { type: "start" });
  expect(s).toEqual({ running: true, step: 0, cycle: 0, done: false });
  for (let i = 0; i < n - 1; i++) s = breathReducer(s, { type: "advance", stepsPerCycle: n });
  expect(s.step).toBe(n - 1);
  s = breathReducer(s, { type: "advance", stepsPerCycle: n });
  expect(s).toEqual({ running: true, step: 0, cycle: 1, done: false });
  for (let i = 0; i < n * (BREATH_CYCLES - 1); i++) {
    s = breathReducer(s, { type: "advance", stepsPerCycle: n });
  }
  expect(s).toEqual({ running: false, step: 0, cycle: BREATH_CYCLES, done: true });
  expect(breathReducer(s, { type: "stop" })).toEqual(idle);
});

it("advance diabaikan bila tidak sedang berjalan", () => {
  expect(breathReducer(idle, { type: "advance", stepsPerCycle: 4 })).toEqual(idle);
});

it("skala bulatan: tarik napas besar, hembus kecil, tahan mengikuti langkah sebelumnya", () => {
  const box = PATTERNS.box;
  expect(scaleAfter(box, 0)).toBe(1);
  expect(scaleAfter(box, 1)).toBe(1);
  expect(scaleAfter(box, 2)).toBe(SMALL);
  expect(scaleAfter(box, 3)).toBe(SMALL);
  expect(scaleAfter(PATTERNS.calm, 1)).toBe(1);
});
