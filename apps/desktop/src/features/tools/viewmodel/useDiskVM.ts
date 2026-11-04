import { useCallback, useEffect, useState } from "react";
import { commands, type DiskStatusDto } from "../../../shared/ipc";

const POLL_MS = 500;

/** Ukuran berkas ringkas: 1,5 GB, 320 MB, 12 KB. Titik desimal netral (tanpa locale). */
export function formatBytes(n: number): string {
  if (n < 1024) return `${Math.max(0, Math.round(n))} B`;
  const units = ["KB", "MB", "GB", "TB", "PB"];
  let v = n;
  let i = -1;
  do {
    v /= 1024;
    i++;
  } while (v >= 1024 && i < units.length - 1);
  return `${v >= 100 ? v.toFixed(0) : v >= 10 ? v.toFixed(1) : v.toFixed(2)} ${units[i]}`;
}

/** Lebar batang 0-100 relatif terhadap total folder; tidak pernah keluar rentang. */
export function sharePercent(size: number, total: number): number {
  return total > 0 ? Math.min(100, Math.max(0, (size / total) * 100)) : 0;
}

/** Pemindai ukuran folder: pilih drive, masuk folder, naik; hasil di-poll selama pemindaian berjalan. */
export function useDiskVM() {
  const [drives, setDrives] = useState<string[]>([]);
  const [status, setStatus] = useState<DiskStatusDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  const poll = useCallback(async () => setStatus(await commands.diskStatus()), []);

  useEffect(() => {
    void commands.diskDrives().then(setDrives);
    void poll(); // hasil pemindaian terakhir tetap ada di backend saat panel dibuka lagi
  }, [poll]);

  const running = status?.running === true;
  useEffect(() => {
    if (!running) return;
    const id = window.setInterval(() => void poll(), POLL_MS);
    return () => window.clearInterval(id);
  }, [running, poll]);

  return {
    drives,
    status,
    error: error ?? status?.error ?? null,
    scan: async (path: string) => {
      const r = await commands.diskScan(path);
      setError(r.status === "error" ? r.error : null);
      await poll();
    },
    reveal: async (path: string) => {
      const r = await commands.diskReveal(path);
      setError(r.status === "error" ? r.error : null);
    },
  };
}
