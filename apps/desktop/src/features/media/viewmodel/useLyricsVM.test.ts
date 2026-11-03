import { describe, expect, it } from "vitest";
import { currentLyric, lineIndexAt } from "./useLyricsVM";

const lines = [
  { ms: 1000, text: "satu" },
  { ms: 5000, text: "dua" },
  { ms: 9000, text: "" },
  { ms: 12000, text: "tiga" },
];

describe("lirik", () => {
  it("lineIndexAt mencari baris aktif dengan pencarian biner", () => {
    expect(lineIndexAt(lines, 0)).toBe(-1);
    expect(lineIndexAt(lines, 1000)).toBe(0);
    expect(lineIndexAt(lines, 4999)).toBe(0);
    expect(lineIndexAt(lines, 5000)).toBe(1);
    expect(lineIndexAt(lines, 999_999)).toBe(3);
    expect(lineIndexAt([], 100)).toBe(-1);
  });

  it("currentLyric kosong sebelum lirik mulai dan saat jeda instrumental", () => {
    expect(currentLyric(lines, 500)).toBeNull();
    expect(currentLyric(lines, 6000)?.text).toBe("dua");
    expect(currentLyric(lines, 10_000)).toBeNull();
    expect(currentLyric(lines, 13_000)?.text).toBe("tiga");
  });

  it("fraction naik dari 0 ke 1 sepanjang baris lalu tertahan", () => {
    expect(currentLyric(lines, 1000)?.fraction).toBe(0);
    const mid = currentLyric(lines, 2000)?.fraction ?? -1;
    expect(mid).toBeGreaterThan(0);
    expect(mid).toBeLessThan(1);
    expect(currentLyric(lines, 4800)?.fraction).toBe(1);
    // baris terakhir tanpa baris sesudahnya tetap punya batas wajar
    expect(currentLyric(lines, 60_000)?.fraction).toBe(1);
  });
});
