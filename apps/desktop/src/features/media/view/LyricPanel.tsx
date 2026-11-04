import type { LyricLineDto } from "../../../shared/ipc";
import { currentLyric, lineIndexAt } from "../viewmodel/useLyricsVM";

/**
 * Kolom lirik di kanan kartu media: baris yang sedang dinyanyikan (boleh turun beberapa baris,
 * kata yang sudah dinyanyikan berwarna aksen) dan satu baris berikutnya yang redup.
 */
export function LyricPanel({ lines, positionMs }: { lines: LyricLineDto[]; positionMs: number }) {
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
    </div>
  );
}
