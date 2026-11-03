import { useCallback, useEffect, useState } from "react";
import { commands, type PomodoroStatusDto } from "../../../shared/ipc";

const POLL_MS = 1000;

/** `m:ss`, atau `j:mm:ss` bila lebih dari satu jam. */
export function formatRemaining(secs: number): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return h > 0 ? `${h}:${pad(m)}:${pad(secs % 60)}` : `${m}:${pad(secs % 60)}`;
}

export const MAX_FOCUS_SECS = 24 * 3600;

/** Jam, menit, detik dari kotak input menjadi total detik; tidak valid (nol atau negatif) jadi 0, lebih dari 24 jam dijepit. */
export function toSeconds(h: number, m: number, s: number): number {
  const part = (v: number) => (Number.isFinite(v) && v > 0 ? Math.floor(v) : 0);
  return Math.min(MAX_FOCUS_SECS, part(h) * 3600 + part(m) * 60 + part(s));
}

/** Status Pomodoro dari backend (diperiksa tiap detik selama jendela Setelan terbuka). */
export function usePomodoroVM() {
  const [status, setStatus] = useState<PomodoroStatusDto>({ phase: "idle", remaining_secs: 0 });
  const refresh = useCallback(() => void commands.pomodoroStatus().then(setStatus), []);

  useEffect(() => {
    refresh();
    const timer = setInterval(refresh, POLL_MS);
    return () => clearInterval(timer);
  }, [refresh]);

  return {
    status,
    /** `seconds` 0 = memakai lama fokus dari setelan. */
    start: (seconds = 0) => void commands.pomodoroStart(seconds).then(refresh),
    stop: () => void commands.pomodoroStop().then(refresh),
  };
}
