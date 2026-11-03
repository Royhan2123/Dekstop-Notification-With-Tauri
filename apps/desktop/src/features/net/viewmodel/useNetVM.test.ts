import { describe, expect, it } from "vitest";
import { formatRate, sparkline } from "./useNetVM";

describe("jaringan", () => {
  it("formatRate memakai satuan seperti di ponsel", () => {
    expect(formatRate(0)).toBe("0 B/s");
    expect(formatRate(820)).toBe("820 B/s");
    expect(formatRate(52 * 1024)).toBe("52 KB/s");
    expect(formatRate(1.5 * 1024 * 1024)).toBe("1.5 MB/s");
  });

  it("sparkline relatif ke puncak dan aman saat semuanya nol", () => {
    expect(sparkline([0, 50, 100])).toEqual([0, 50, 100]);
    expect(sparkline([0, 0])).toEqual([0, 0]);
    expect(sparkline([])).toEqual([]);
  });
});
