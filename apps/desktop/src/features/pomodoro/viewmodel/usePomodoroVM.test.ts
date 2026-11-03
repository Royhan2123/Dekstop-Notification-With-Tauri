import { describe, expect, it } from "vitest";
import { formatRemaining, toSeconds } from "./usePomodoroVM";

describe("pomodoro", () => {
  it("formatRemaining: m:ss, dan j:mm:ss bila lewat sejam", () => {
    expect(formatRemaining(0)).toBe("0:00");
    expect(formatRemaining(65)).toBe("1:05");
    expect(formatRemaining(3600)).toBe("1:00:00");
    expect(formatRemaining(3725)).toBe("1:02:05");
  });

  it("toSeconds menjumlah jam, menit, detik; tak valid jadi nol; dijepit 24 jam", () => {
    expect(toSeconds(0, 25, 0)).toBe(1500);
    expect(toSeconds(1, 2, 3)).toBe(3723);
    expect(toSeconds(0, 0, 0)).toBe(0);
    expect(toSeconds(-5, Number.NaN, 0)).toBe(0);
    expect(toSeconds(99, 0, 0)).toBe(24 * 3600);
  });
});
