/** Port yang dicoba berurutan; harus sama dengan PORT_RANGE di aplikasi. */
export const PORTS = Array.from({ length: 10 }, (_, i) => 47821 + i);

export type Command = { type: "set_rate"; rate: number } | { type: "search"; query: string };

export function clampRate(rate: number): number | null {
  if (!Number.isFinite(rate)) return null;
  return Math.round(Math.min(2, Math.max(0.25, rate)) * 20) / 20;
}

export function searchUrl(query: string): string {
  return `https://www.youtube.com/results?search_query=${encodeURIComponent(query)}`;
}

/** Pesan dari aplikasi; apa pun yang tidak dikenal atau tidak sah diabaikan. */
export function parseCommand(raw: string): Command | null {
  let msg: unknown;
  try {
    msg = JSON.parse(raw);
  } catch {
    return null;
  }
  if (typeof msg !== "object" || msg === null) return null;
  const m = msg as Record<string, unknown>;
  if (m.type === "set_rate" && typeof m.rate === "number") {
    const rate = clampRate(m.rate);
    return rate === null ? null : { type: "set_rate", rate };
  }
  if (m.type === "search" && typeof m.query === "string" && m.query.trim()) {
    return { type: "search", query: m.query.trim().slice(0, 200) };
  }
  return null;
}

/** Tab yang sedang bersuara diutamakan; kalau tidak ada, tab pertama. */
export function pickTab<T extends { audible?: boolean }>(tabs: T[]): T | undefined {
  return tabs.find((t) => t.audible) ?? tabs[0];
}
