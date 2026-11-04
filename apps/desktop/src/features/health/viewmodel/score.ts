/** Hari beruntun (dari yang terbaru mundur) dengan waktu layar di bawah batas; berhenti di hari pertama yang melewatinya. */
export function streakDays(minutesAsc: number[], goalMin: number): number {
  let n = 0;
  for (let i = minutesAsc.length - 1; i >= 0 && minutesAsc[i] < goalMin; i--) n++;
  return n;
}

export interface ScoreInput {
  todayMin: number;
  goalMin: number;
  water: number;
  waterGoal: number;
  eyes: number;
  eyeGoal: number;
  stretch: number;
  stretchGoal: number;
}

const ratio = (v: number, goal: number) => (goal > 0 ? Math.min(1, Math.max(0, v / goal)) : 0);

/** Skor hari ini 0-100: waktu layar 40, air 30, istirahat mata 15, peregangan 15. */
export function dayScore(i: ScoreInput): number {
  return Math.round(
    40 * (1 - ratio(i.todayMin, i.goalMin)) +
      30 * ratio(i.water, i.waterGoal) +
      15 * ratio(i.eyes, i.eyeGoal) +
      15 * ratio(i.stretch, i.stretchGoal),
  );
}
