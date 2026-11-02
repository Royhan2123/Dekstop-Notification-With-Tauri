import { useEffect, useState } from "react";
import { commands, type DayUsageDto, type HealthSummaryDto } from "../../../shared/ipc";

export interface Bar {
  date: string;
  minutes: number;
  /** 0-100, relatif terhadap hari terpanjang. */
  percent: number;
}

/** Tinggi batang grafik; hari tanpa data tetap 0, bukan NaN. */
export function chartBars(days: DayUsageDto[]): Bar[] {
  const max = Math.max(1, ...days.map((d) => d.active_min));
  return days.map((d) => ({
    date: d.date,
    minutes: d.active_min,
    percent: Math.round((d.active_min / max) * 100),
  }));
}

export function formatHours(minutes: number): string {
  return `${Math.floor(minutes / 60)}:${String(minutes % 60).padStart(2, "0")}`;
}

/** Rata-rata kecerahan hanya dari hari yang punya data; `null` bila belum ada sama sekali. */
export function averageBrightness(days: DayUsageDto[]): number | null {
  const known = days.flatMap((d) => (d.avg_brightness === null ? [] : [d.avg_brightness]));
  return known.length ? Math.round(known.reduce((a, b) => a + b, 0) / known.length) : null;
}

export function useHealthVM() {
  const [summary, setSummary] = useState<HealthSummaryDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void commands.healthSummary().then((r) => {
      if (r.status === "ok") setSummary(r.data);
      else setError(r.error);
    });
  }, []);

  return {
    summary,
    error,
    bars: summary ? chartBars(summary.days) : [],
    weekBrightness: summary ? averageBrightness(summary.days) : null,
  };
}
