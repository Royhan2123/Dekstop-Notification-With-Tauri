import { useEffect, useReducer, useState } from "react";

export type BreathKind = "inhale" | "hold" | "exhale";
export interface BreathStep {
  kind: BreathKind;
  secs: number;
}
export type PatternId = "box" | "calm";

export const PATTERNS: Record<PatternId, BreathStep[]> = {
  box: [
    { kind: "inhale", secs: 4 },
    { kind: "hold", secs: 4 },
    { kind: "exhale", secs: 4 },
    { kind: "hold", secs: 4 },
  ],
  calm: [
    { kind: "inhale", secs: 4 },
    { kind: "hold", secs: 7 },
    { kind: "exhale", secs: 8 },
  ],
};

/** Jumlah siklus satu sesi. */
export const BREATH_CYCLES = 4;
const DONE_MS = 2500;
export const SMALL = 0.55;

export interface BreathState {
  running: boolean;
  step: number;
  cycle: number;
  done: boolean;
}

export type BreathEvent =
  { type: "start" } | { type: "advance"; stepsPerCycle: number } | { type: "stop" };

const IDLE: BreathState = { running: false, step: 0, cycle: 0, done: false };

export function breathReducer(s: BreathState, e: BreathEvent): BreathState {
  switch (e.type) {
    case "start":
      return { running: true, step: 0, cycle: 0, done: false };
    case "stop":
      return IDLE;
    case "advance": {
      if (!s.running) return s;
      if (s.step + 1 < e.stepsPerCycle) return { ...s, step: s.step + 1 };
      const cycle = s.cycle + 1;
      return cycle >= BREATH_CYCLES
        ? { running: false, step: 0, cycle, done: true }
        : { ...s, step: 0, cycle };
    }
  }
}

/** Skala bulatan di akhir langkah: tarik napas membesar, hembus mengecil, tahan mempertahankan. */
export function scaleAfter(steps: BreathStep[], index: number): number {
  for (let i = index; i >= 0; i--) {
    if (steps[i].kind === "inhale") return 1;
    if (steps[i].kind === "exhale") return SMALL;
  }
  return SMALL;
}

/** Sesi latihan napas berpemandu; `onDone` dipanggil sekali saat semua siklus selesai. */
export function useBreathVM(onDone: () => void) {
  const [pattern, setPattern] = useState<PatternId>("box");
  const [state, dispatch] = useReducer(breathReducer, IDLE);
  const steps = PATTERNS[pattern];

  useEffect(() => {
    if (!state.running) return;
    const id = window.setTimeout(
      () => dispatch({ type: "advance", stepsPerCycle: steps.length }),
      steps[state.step].secs * 1000,
    );
    return () => window.clearTimeout(id);
  }, [state, steps]);

  useEffect(() => {
    if (!state.done) return;
    onDone();
    const id = window.setTimeout(() => dispatch({ type: "stop" }), DONE_MS);
    return () => window.clearTimeout(id);
    // onDone sengaja tidak jadi dependensi: hanya reaksi pada selesainya sesi.
  }, [state.done]);

  return {
    pattern,
    setPattern: (p: PatternId) => {
      setPattern(p);
      dispatch({ type: "stop" });
    },
    steps,
    state,
    current: steps[state.step],
    start: () => dispatch({ type: "start" }),
    stop: () => dispatch({ type: "stop" }),
  };
}
