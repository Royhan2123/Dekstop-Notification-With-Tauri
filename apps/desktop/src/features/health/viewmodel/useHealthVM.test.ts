import { expect, it } from "vitest";
import type { DayUsageDto } from "../../../shared/ipc";
import { averageBrightness, chartBars, formatHours } from "./useHealthVM";

const day = (
  date: string,
  active_min: number,
  avg_brightness: number | null = null,
): DayUsageDto => ({ date, active_min, avg_brightness });

it("batang relatif terhadap hari terpanjang dan hari kosong 0", () => {
  const bars = chartBars([day("2026-11-01", 0), day("2026-11-02", 60), day("2026-11-03", 240)]);
  expect(bars.map((b) => b.percent)).toEqual([0, 25, 100]);
  expect(chartBars([day("2026-11-01", 0)])[0].percent).toBe(0);
});

it("jam diformat dan rata-rata kecerahan mengabaikan hari tanpa data", () => {
  expect(formatHours(125)).toBe("2:05");
  expect(averageBrightness([day("a", 1, 60), day("b", 1, null), day("c", 1, 80)])).toBe(70);
  expect(averageBrightness([day("a", 1)])).toBeNull();
});
