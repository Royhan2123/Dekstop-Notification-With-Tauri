import { expect, it } from "vitest";
import { approach, NEUTRAL, poseAt, type PoseMode } from "./pose";

const MODES: PoseMode[] = ["idle", "neck", "shoulders", "wrists", "back", "done"];

it("setiap gerakan menghasilkan sudut yang terbatas dan wajar di sepanjang waktu", () => {
  for (const mode of MODES) {
    for (let t = 0; t < 30; t += 0.1) {
      const p = poseAt(mode, t);
      for (const [k, v] of Object.entries(p)) {
        expect(Number.isFinite(v), `${mode}.${k}`).toBe(true);
        // Putaran pergelangan boleh bertambah terus; sudut lain tidak melewati satu putaran penuh.
        if (k !== "wristL" && k !== "wristR") expect(Math.abs(v), `${mode}.${k}`).toBeLessThan(3.2);
      }
    }
  }
});

it("gerakan berulang mulus: pose pada awal dan akhir satu siklus sama", () => {
  const cycles: [PoseMode, number][] = [
    ["neck", 4],
    ["shoulders", 2.6],
    ["back", 8],
    ["done", 1.8],
  ];
  for (const [mode, period] of cycles) {
    const a = poseAt(mode, 0.37);
    const b = poseAt(mode, 0.37 + period);
    for (const k of Object.keys(a) as (keyof typeof a)[]) {
      if (k === "bob") continue; // napas punya periode sendiri (3,2 dtk)
      expect(b[k]).toBeCloseTo(a[k], 6);
    }
  }
});

it("tiap gerakan memang menggerakkan bagian tubuh yang dimaksud", () => {
  const range = (mode: PoseMode, key: keyof typeof NEUTRAL) => {
    const v = Array.from({ length: 80 }, (_, i) => poseAt(mode, i * 0.1)[key]);
    return Math.max(...v) - Math.min(...v);
  };
  expect(range("neck", "headZ")).toBeGreaterThan(0.6);
  expect(range("shoulders", "shrug")).toBeGreaterThan(0.1);
  expect(range("wrists", "wristL")).toBeGreaterThan(6);
  expect(poseAt("back", 1).armLZ).toBeGreaterThan(2.4); // tangan terangkat tinggi
  expect(poseAt("idle", 1).armLZ).toBeLessThan(0.5); // tangan turun
});

it("peralihan antar-pose mendekati target tanpa melompat dan tidak melampauinya", () => {
  const target = poseAt("back", 1);
  let cur = { ...NEUTRAL };
  const first = approach(cur, target, 1 / 60);
  expect(first.armLZ).toBeGreaterThan(cur.armLZ);
  expect(first.armLZ).toBeLessThan(target.armLZ);
  for (let i = 0; i < 120; i++) cur = approach(cur, target, 1 / 60);
  expect(cur.armLZ).toBeCloseTo(target.armLZ, 2);
  expect(approach(cur, target, 0)).toEqual(cur);
});
