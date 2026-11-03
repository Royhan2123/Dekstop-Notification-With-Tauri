import { useEffect, useState } from "react";
import { commands, type LyricLineDto } from "../../../shared/ipc";
import { useIslandStore } from "../../island/store";

/** Jeda sebelum mencoba lagi bila server lirik sibuk atau tanpa internet, dan batas percobaannya. */
const RETRY_MS = 15_000;
const MAX_RETRIES = 3;

/** Indeks baris yang sedang dinyanyikan pada `positionMs`; -1 sebelum baris pertama. */
export function lineIndexAt(lines: LyricLineDto[], positionMs: number): number {
  let lo = 0;
  let hi = lines.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (lines[mid].ms <= positionMs) lo = mid + 1;
    else hi = mid;
  }
  return lo - 1;
}

/** Teks baris aktif; baris kosong (jeda instrumental) dan sebelum lirik dimulai menghasilkan `null`. */
export function currentLyric(lines: LyricLineDto[], positionMs: number): string | null {
  const i = lineIndexAt(lines, positionMs);
  return i >= 0 && lines[i].text ? lines[i].text : null;
}

/** Lirik berwaktu lagu yang sedang diputar; kosong bila tidak ada. Diambil sekali per lagu. */
export function useLyricsVM(): LyricLineDto[] {
  const content = useIslandStore((s) => s.snapshot.content);
  const duration = useIslandStore((s) => s.snapshot.media?.duration_ms ?? 0);
  const title = content?.kind === "media" ? content.text : "";
  const artist = content?.kind === "media" ? (content.detail ?? "") : "";
  const [lines, setLines] = useState<LyricLineDto[]>([]);

  useEffect(() => {
    setLines([]);
    if (!title || duration <= 0) return;
    let alive = true;
    let timer: number | undefined;
    const attempt = (n: number) => {
      void commands.lyricsFetch(artist, title, duration).then((r) => {
        if (!alive) return;
        if (r.status === "ok") setLines(r.data);
        else if (n < MAX_RETRIES) timer = window.setTimeout(() => attempt(n + 1), RETRY_MS);
      });
    };
    attempt(1);
    return () => {
      alive = false;
      window.clearTimeout(timer);
    };
  }, [artist, title, Math.round(duration / 1000)]);

  return lines;
}
