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
    expect(currentLyric(lines, 6000)).toBe("dua");
    expect(currentLyric(lines, 10_000)).toBeNull();
    expect(currentLyric(lines, 13_000)).toBe("tiga");
  });
});
