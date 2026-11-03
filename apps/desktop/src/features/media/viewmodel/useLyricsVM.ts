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

/** Perkiraan lama menyanyikan satu huruf; sumber lirik hanya punya waktu per baris, bukan per kata. */
const MS_PER_CHAR = 110;
const MIN_SING_MS = 1500;

/**
 * Baris aktif dan seberapa jauh (0..1) ia sudah dinyanyikan. Baris kosong (jeda instrumental) dan
 * sebelum lirik dimulai menghasilkan `null`.
 * shortcut: kemajuan dalam baris diperkirakan dari jumlah huruf, bukan waktu tiap kata; upgrade bila ada sumber lirik per kata.
 */
export function currentLyric(
  lines: LyricLineDto[],
  positionMs: number,
): { text: string; fraction: number } | null {
  const i = lineIndexAt(lines, positionMs);
  if (i < 0 || !lines[i].text) return null;
  const { ms, text } = lines[i];
  const next = lines[i + 1]?.ms ?? ms + MIN_SING_MS * 3;
  const sing = Math.max(
    300,
    Math.min(next - 100 - ms, Math.max(MIN_SING_MS, [...text].length * MS_PER_CHAR)),
  );
  return { text, fraction: Math.min(1, Math.max(0, (positionMs - ms) / sing)) };
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
