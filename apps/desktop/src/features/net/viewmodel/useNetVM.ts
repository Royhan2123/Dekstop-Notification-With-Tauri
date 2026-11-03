import { useEffect, useState } from "react";
import { commands } from "../../../shared/ipc";

export const HISTORY = 30;

/** Kecepatan seperti di ponsel: `820 B/s`, `52 KB/s`, `1.4 MB/s`. */
export function formatRate(bps: number): string {
  if (bps < 1024) return `${bps} B/s`;
  if (bps < 1024 * 1024) return `${Math.round(bps / 1024)} KB/s`;
  return `${(bps / 1024 / 1024).toFixed(1)} MB/s`;
}

/** Tinggi batang 0-100 relatif terhadap puncak riwayat; riwayat sepi tidak jadi NaN. */
export function sparkline(values: number[]): number[] {
  const max = Math.max(1, ...values);
  return values.map((v) => Math.round((v / max) * 100));
}

/** Memantau kecepatan tiap detik selama panel terbuka; riwayat 30 detik terakhir untuk grafik mini. */
export function useNetVM() {
  const [down, setDown] = useState<number[]>([]);
  const [up, setUp] = useState<number[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    const poll = async () => {
      const r = await commands.netRate();
      if (!alive) return;
      if (r.status === "ok") {
        setError(null);
        setDown((h) => [...h, r.data.down_bps].slice(-HISTORY));
        setUp((h) => [...h, r.data.up_bps].slice(-HISTORY));
      } else {
        setError(r.error);
      }
    };
    void poll();
    const id = window.setInterval(() => void poll(), 1000);
    return () => {
      alive = false;
      window.clearInterval(id);
    };
  }, []);

  return { down, up, error };
}
