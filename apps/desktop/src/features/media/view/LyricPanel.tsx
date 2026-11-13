import { useT } from "../../../shared/i18n";
import type { LyricLineDto } from "../../../shared/ipc";
import { currentLyric, lineIndexAt } from "../viewmodel/useLyricsVM";

/** Langkah tombol geser lirik (ms). */
export const SHIFT_STEP_MS = 250;

/** "+0,5 dtk" / "-1 dtk" / "0": selisih lirik terhadap lagu. */
export function formatOffset(ms: number): string {
  if (ms === 0) return "0";
  const secs = Math.abs(ms) / 1000;
  const text = (Number.isInteger(secs) ? String(secs) : secs.toFixed(2).replace(/0$/, "")).replace(
    ".",
    ",",
  );
  return `${ms > 0 ? "+" : "-"}${text}s`;
}

/**
 * Kolom lirik di kanan kartu media: baris yang sedang dinyanyikan (boleh turun beberapa baris,
 * kata yang sudah dinyanyikan berwarna aksen) dan satu baris berikutnya yang redup.
 */
export function LyricPanel({
  lines,
  positionMs,
  offsetMs,
  onShift,
}: {
  lines: LyricLineDto[];
  /** Posisi lagu yang sebenarnya; pergeseran `offsetMs` ditambahkan di sini. */
  positionMs: number;
  offsetMs: number;
  onShift: (deltaMs: number) => void;
}) {
  const t = useT();
  positionMs += offsetMs;
  const now = currentLyric(lines, positionMs);
  const words = now ? now.text.split(/\s+/).filter(Boolean) : [];
  const next = lines[lineIndexAt(lines, positionMs) + 1]?.text;
  return (
    <div className="lyric-panel">
      <p className="lyric-now">
        {words.map((w, i) => (
          <span key={i} className={i / words.length < (now?.fraction ?? 0) ? "on" : undefined}>
            {w}{" "}
          </span>
        ))}
      </p>
      {next && <p className="lyric-next">{next}</p>}
      <div className="lyric-shift" role="group" aria-label={t("lyrics.shift")}>
        <button type="button" title={t("lyrics.later")} onClick={() => onShift(-SHIFT_STEP_MS)}>
          −
        </button>
        <span>{formatOffset(offsetMs)}</span>
        <button type="button" title={t("lyrics.earlier")} onClick={() => onShift(SHIFT_STEP_MS)}>
          +
        </button>
      </div>
    </div>
  );
}
