import { useCallback, useEffect, useState } from "react";
import { commands, type PomodoroStatusDto } from "../../../shared/ipc";

const POLL_MS = 1000;

export function formatRemaining(secs: number): string {
  return `${Math.floor(secs / 60)}:${String(secs % 60).padStart(2, "0")}`;
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
    start: () => void commands.pomodoroStart().then(refresh),
    stop: () => void commands.pomodoroStop().then(refresh),
  };
}
