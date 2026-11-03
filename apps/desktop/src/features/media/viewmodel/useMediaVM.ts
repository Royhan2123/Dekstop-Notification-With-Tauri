import { useEffect, useRef, useState } from "react";
import { commands, type MediaDto } from "../../../shared/ipc";
import { useIslandStore } from "../../island/store";

const TICK_MS = 250;
/** Saat lirik tampil, warna karaoke butuh pembaruan lebih rapat agar bergerak halus. */
export const LYRIC_TICK_MS = 100;
const WHEEL_COOLDOWN_MS = 300;

/** Posisi (ms) pada waktu `now`: posisi terakhir + waktu berjalan x kecepatan bila diputar. */
export function progressAt(m: MediaDto, receivedAt: number, now: number): number {
  const elapsed = m.playing ? Math.max(0, now - receivedAt) * m.rate : 0;
  const pos = m.position_ms + elapsed;
  return m.duration_ms > 0 ? Math.min(pos, m.duration_ms) : pos;
}

export function formatTime(ms: number): string {
  const total = Math.floor(ms / 1000);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

/** `ticking`: perbarui progress terus-menerus (saat tampilan lebar atau ada lirik, agar CPU idle tetap rendah). */
export function useMediaVM(ticking: boolean, tickMs = TICK_MS) {
  const media = useIslandStore((s) => s.snapshot.media);
  const receivedAt = useIslandStore((s) => s.receivedAt);
  const extensionConnected = useIslandStore((s) => s.snapshot.extension_connected);
  const [now, setNow] = useState(() => Date.now());
  const lastWheel = useRef(0);

  const running = ticking && media?.playing === true;
  useEffect(() => {
    if (!running) return;
    const timer = setInterval(() => setNow(Date.now()), tickMs);
    return () => clearInterval(timer);
  }, [running, tickMs]);

  return {
    media,
    extensionConnected,
    onRate: (rate: number) => void commands.bridgeSetRate(rate),
    onSearch: (query: string) => void commands.bridgeSearch(query),
    positionMs: media ? progressAt(media, receivedAt, running ? now : receivedAt) : 0,
    onPrevious: () => void commands.mediaControl("previous"),
    onToggle: () => void commands.mediaControl("toggle_play"),
    onNext: () => void commands.mediaControl("next"),
    onWheel: (deltaY: number) => {
      const t = Date.now();
      if (deltaY === 0 || t - lastWheel.current < WHEEL_COOLDOWN_MS) return;
      lastWheel.current = t;
      void commands.mediaCycle(deltaY > 0 ? 1 : -1);
    },
  };
}
