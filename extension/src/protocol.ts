/** Port yang dicoba berurutan; harus sama dengan PORT_RANGE di aplikasi. */
export const PORTS = Array.from({ length: 10 }, (_, i) => 47821 + i);

export type MediaAction = "toggle" | "next" | "previous";

export type Command =
  | { type: "set_rate"; rate: number }
  | { type: "search"; query: string }
  | { type: "media_control"; tabId: number; action: MediaAction };

/** Lagu/video yang sedang diputar di satu tab, dilaporkan content script. */
export interface MediaState {
  title: string;
  artist: string;
  playing: boolean;
  positionMs: number;
  durationMs: number;
  rate: number;
  artwork: string;
}

const MAX_SESSIONS = 8;
const MAX_TEXT = 200;

export function sessionId(tabId: number): string {
  return `browser:${tabId}`;
}

/** Kebalikan `sessionId`; null bila bukan sesi browser. */
export function tabOfSession(id: string): number | null {
  const m = /^browser:(\d+)$/.exec(id);
  return m ? Number(m[1]) : null;
}

const text = (v: unknown): string =>
  typeof v === "string"
    ? v
        .replace(/[\u0000-\u001f\u007f]/g, " ")
        .trim()
        .slice(0, MAX_TEXT)
    : "";
const num = (v: unknown, max: number): number =>
  typeof v === "number" && Number.isFinite(v) ? Math.min(Math.max(v, 0), max) : 0;

/** Pesan dari content script dianggap tidak dipercaya: teks dirapikan, angka dibatasi, judul wajib. */
export function sanitizeState(raw: unknown): MediaState | null {
  if (typeof raw !== "object" || raw === null) return null;
  const s = raw as Record<string, unknown>;
  const title = text(s.title);
  if (!title) return null;
  const rate = typeof s.rate === "number" ? clampRate(s.rate) : null;
  return {
    title,
    artist: text(s.artist),
    playing: s.playing === true,
    positionMs: num(s.positionMs, 86_400_000),
    durationMs: num(s.durationMs, 86_400_000),
    rate: rate ?? 1,
    artwork: typeof s.artwork === "string" ? s.artwork.trim().slice(0, 500) : "",
  };
}

/** Pesan `media` ke aplikasi: semua tab yang sedang punya media (maks 8). */
export function mediaMessage(states: ReadonlyMap<number, MediaState>) {
  const sessions = [...states].slice(0, MAX_SESSIONS).map(([tabId, s]) => ({
    id: sessionId(tabId),
    title: s.title,
    artist: s.artist,
    playing: s.playing,
    position_ms: s.positionMs,
    duration_ms: s.durationMs,
    rate: s.rate,
    artwork: s.artwork,
  }));
  return { type: "media" as const, sessions };
}

/** Sampul terbesar dari daftar `MediaMetadata.artwork` (sizes "WxH"); kosong bila tak ada. */
export function pickArtwork(list: ReadonlyArray<{ src: string; sizes?: string }>): string {
  const area = (sizes?: string) => {
    const m = /(\d+)x(\d+)/.exec(sizes ?? "");
    return m ? Number(m[1]) * Number(m[2]) : 0;
  };
  const best = [...list].sort((a, b) => area(b.sizes) - area(a.sizes))[0];
  return best?.src ?? "";
}

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
  if (m.type === "media_control" && typeof m.id === "string") {
    const tabId = tabOfSession(m.id);
    const action = m.action;
    if (tabId !== null && (action === "toggle" || action === "next" || action === "previous")) {
      return { type: "media_control", tabId, action };
    }
  }
  return null;
}

/** Tab yang sedang bersuara diutamakan; kalau tidak ada, tab pertama. */
export function pickTab<T extends { audible?: boolean }>(tabs: T[]): T | undefined {
  return tabs.find((t) => t.audible) ?? tabs[0];
}
