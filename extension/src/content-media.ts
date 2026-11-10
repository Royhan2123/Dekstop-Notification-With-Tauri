// Content script situs media (YouTube, YT Music, Spotify Web, ...): laporkan lagu/video yang sedang
// diputar ke ekstensi dan jalankan kontrol dari pulau Isla. Tanpa import: content script MV3 bukan modul.
type Action = "toggle" | "next" | "previous";

interface State {
  title: string;
  artist: string;
  playing: boolean;
  positionMs: number;
  durationMs: number;
  rate: number;
  artwork: string;
}

/** Tombol "berikutnya/sebelumnya" milik masing-masing situs; tanpa ini kontrol itu diabaikan. */
const BUTTONS: Record<string, { next: string; previous: string }> = {
  "www.youtube.com": { next: ".ytp-next-button", previous: ".ytp-prev-button" },
  "music.youtube.com": { next: ".next-button", previous: ".previous-button" },
  "open.spotify.com": {
    next: '[data-testid="control-button-skip-forward"]',
    previous: '[data-testid="control-button-skip-back"]',
  },
  "soundcloud.com": { next: ".skipControl__next", previous: ".skipControl__previous" },
  "www.deezer.com": {
    next: '[data-testid="next_track_button"]',
    previous: '[data-testid="previous_track_button"]',
  },
};

const TICK_MS = 1000;
const KEEPALIVE_MS = 10_000;
const SEEK_TOLERANCE_MS = 1500;

function mediaElement(): HTMLMediaElement | null {
  const all = Array.from(document.querySelectorAll<HTMLMediaElement>("video, audio")).filter(
    (m) => m.readyState > 0 && Number.isFinite(m.duration) && m.duration > 0,
  );
  return all.find((m) => !m.paused) ?? all[0] ?? null;
}

function biggest(art: readonly MediaImage[]): string {
  const area = (s?: string) => {
    const m = /(\d+)x(\d+)/.exec(s ?? "");
    return m ? Number(m[1]) * Number(m[2]) : 0;
  };
  return [...art].sort((a, b) => area(b.sizes) - area(a.sizes))[0]?.src ?? "";
}

function read(): State | null {
  const el = mediaElement();
  if (!el) return null;
  const meta = navigator.mediaSession?.metadata;
  const title = meta?.title || document.title.replace(/\s[-–]\sYouTube.*$/, "").trim();
  if (!title) return null;
  return {
    title,
    artist: meta?.artist || location.hostname.replace(/^www\./, ""),
    playing: !el.paused && !el.ended,
    positionMs: Math.round(el.currentTime * 1000),
    durationMs: Math.round(el.duration * 1000),
    rate: el.playbackRate,
    artwork: biggest(meta?.artwork ?? []),
  };
}

let last: State | null = null;
let lastAt = 0;

function differs(a: State, b: State, elapsedMs: number): boolean {
  const expected = a.positionMs + (a.playing ? elapsedMs * a.rate : 0);
  return (
    a.title !== b.title ||
    a.artist !== b.artist ||
    a.playing !== b.playing ||
    a.durationMs !== b.durationMs ||
    a.artwork !== b.artwork ||
    Math.abs(expected - b.positionMs) > SEEK_TOLERANCE_MS
  );
}

function send(state: State | null) {
  try {
    void chrome.runtime.sendMessage({ type: "media_state", state }).catch(() => {});
  } catch {
    // ekstensi dimuat ulang: konteks skrip ini sudah tidak sah
  }
}

function tick() {
  const now = Date.now();
  const state = read();
  if (state === null) {
    if (last !== null) send(null);
    last = null;
    return;
  }
  const elapsed = now - lastAt;
  if (last === null || differs(last, state, elapsed) || elapsed >= KEEPALIVE_MS) {
    send(state);
    last = state;
    lastAt = now;
  }
}

function control(action: Action) {
  const el = mediaElement();
  if (action === "toggle") {
    if (el) void (el.paused ? el.play() : el.pause());
    return;
  }
  const button = BUTTONS[location.hostname]?.[action];
  const target = button ? document.querySelector<HTMLElement>(button) : null;
  if (target) target.click();
  else if (action === "previous" && el) el.currentTime = 0; // tak ada tombol: ulang dari awal
}

chrome.runtime.onMessage.addListener((msg: { type?: string; action?: Action }) => {
  if (msg.type !== "media_control") return;
  if (msg.action === "toggle" || msg.action === "next" || msg.action === "previous") {
    control(msg.action);
    setTimeout(tick, 150); // laporkan hasilnya segera
  }
});

// Peristiwa media tidak menggelembung: didengar di fase capture supaya perubahan terasa instan.
for (const ev of ["play", "pause", "seeked", "loadedmetadata", "ended"]) {
  document.addEventListener(ev, tick, true);
}
setInterval(tick, TICK_MS);
tick();
