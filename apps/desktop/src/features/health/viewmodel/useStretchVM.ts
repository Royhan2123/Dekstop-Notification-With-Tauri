import { useEffect, useReducer } from "react";

/** Rangkaian peregangan: tiap gerakan berlangsung `STRETCH_SECS` detik. */
export const STRETCH_MOVES = ["neck", "shoulders", "wrists", "back"] as const;
export type StretchMove = (typeof STRETCH_MOVES)[number];
export const STRETCH_SECS = 20;
const DONE_MS = 2500;

export interface StretchState {
  running: boolean;
  move: number;
  left: number;
  done: boolean;
}

export type StretchEvent =
  { type: "start" } | { type: "tick" } | { type: "skip" } | { type: "stop" };

const IDLE: StretchState = { running: false, move: 0, left: STRETCH_SECS, done: false };

function next(s: StretchState): StretchState {
  return s.move + 1 >= STRETCH_MOVES.length
    ? { running: false, move: 0, left: STRETCH_SECS, done: true }
    : { ...s, move: s.move + 1, left: STRETCH_SECS };
}

export function stretchReducer(s: StretchState, e: StretchEvent): StretchState {
  switch (e.type) {
    case "start":
      return { running: true, move: 0, left: STRETCH_SECS, done: false };
    case "stop":
      return IDLE;
    case "skip":
      return s.running ? next(s) : s;
    case "tick":
      if (!s.running) return s;
      return s.left > 1 ? { ...s, left: s.left - 1 } : next(s);
  }
}

/** Sesi peregangan berpemandu; `onDone` dipanggil sekali saat semua gerakan selesai. */
export function useStretchVM(onDone: () => void) {
  const [state, dispatch] = useReducer(stretchReducer, IDLE);

  useEffect(() => {
    if (!state.running) return;
    const id = window.setTimeout(() => dispatch({ type: "tick" }), 1000);
    return () => window.clearTimeout(id);
  }, [state]);

  useEffect(() => {
    if (!state.done) return;
    onDone();
    const id = window.setTimeout(() => dispatch({ type: "stop" }), DONE_MS);
    return () => window.clearTimeout(id);
    // onDone sengaja tidak jadi dependensi: hanya reaksi pada selesainya sesi.
  }, [state.done]);

  return {
    state,
    move: STRETCH_MOVES[state.move],
    start: () => dispatch({ type: "start" }),
    skip: () => dispatch({ type: "skip" }),
    stop: () => dispatch({ type: "stop" }),
  };
}
