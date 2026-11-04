import { expect, it } from "vitest";
import { STRETCH_MOVES, STRETCH_SECS, stretchReducer, type StretchState } from "./useStretchVM";

const idle: StretchState = { running: false, move: 0, left: STRETCH_SECS, done: false };

it("tiap gerakan menghitung mundur lalu pindah, dan semua gerakan selesai menandai sesi", () => {
  let s = stretchReducer(idle, { type: "start" });
  for (let i = 0; i < STRETCH_SECS - 1; i++) s = stretchReducer(s, { type: "tick" });
  expect(s).toEqual({ running: true, move: 0, left: 1, done: false });
  s = stretchReducer(s, { type: "tick" });
  expect(s).toEqual({ running: true, move: 1, left: STRETCH_SECS, done: false });
  for (let i = 0; i < (STRETCH_MOVES.length - 1) * STRETCH_SECS; i++) {
    s = stretchReducer(s, { type: "tick" });
  }
  expect(s).toEqual({ ...idle, done: true });
});

it("lewati pindah ke gerakan berikutnya dan di gerakan terakhir menyelesaikan sesi", () => {
  let s = stretchReducer(idle, { type: "start" });
  s = stretchReducer(s, { type: "skip" });
  expect(s.move).toBe(1);
  for (let i = 0; i < STRETCH_MOVES.length - 1; i++) s = stretchReducer(s, { type: "skip" });
  expect(s.done).toBe(true);
  expect(stretchReducer(s, { type: "stop" })).toEqual(idle);
});

it("tick dan lewati diabaikan bila tidak berjalan", () => {
  expect(stretchReducer(idle, { type: "tick" })).toEqual(idle);
  expect(stretchReducer(idle, { type: "skip" })).toEqual(idle);
});
