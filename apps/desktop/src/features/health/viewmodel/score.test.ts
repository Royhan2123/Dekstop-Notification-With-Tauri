import { expect, it } from "vitest";
import { dayScore, streakDays } from "./score";

it("streak menghitung hari terbaru yang di bawah batas dan berhenti di hari yang melewatinya", () => {
  expect(streakDays([], 480)).toBe(0);
  expect(streakDays([100, 200, 300], 480)).toBe(3);
  expect(streakDays([100, 500, 300, 200], 480)).toBe(2);
  expect(streakDays([100, 200, 600], 480)).toBe(0);
});

const base = {
  todayMin: 0,
  goalMin: 480,
  water: 0,
  waterGoal: 8,
  eyes: 0,
  eyeGoal: 6,
  stretch: 0,
  stretchGoal: 3,
};

it("skor 0-100: sempurna 100, layar melewati batas tanpa kebiasaan sehat 0, tidak keluar rentang", () => {
  expect(dayScore({ ...base, water: 8, eyes: 6, stretch: 3 })).toBe(100);
  expect(dayScore({ ...base, todayMin: 600 })).toBe(0);
  expect(dayScore({ ...base, todayMin: 240, water: 4 })).toBe(35);
  expect(dayScore({ ...base, water: 99, eyes: 99, stretch: 99, todayMin: -5 })).toBe(100);
});
