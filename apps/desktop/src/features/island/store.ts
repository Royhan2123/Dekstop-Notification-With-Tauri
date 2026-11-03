import { create } from "zustand";
import type { IslandSnapshot } from "../../shared/ipc";

export const idleSnapshot: IslandSnapshot = {
  mode: "idle",
  content: null,
  media: null,
  review: null,
  mood: "idle",
  extension_connected: false,
  hovered: false,
  shelf: [],
  dnd: false,
};

interface IslandStore {
  snapshot: IslandSnapshot;
  /** Waktu (ms) snapshot diterima; dasar untuk menghitung progress media. */
  receivedAt: number;
  setSnapshot: (s: IslandSnapshot) => void;
}

/** Posisi media baru diukur backend saat membaca sesi, bukan saat snapshot dikirim; lihat `setSnapshot`. */
function sameTimeline(a: IslandSnapshot["media"], b: IslandSnapshot["media"]): boolean {
  return (
    a !== null &&
    b !== null &&
    a.session_index === b.session_index &&
    a.duration_ms === b.duration_ms &&
    a.position_ms === b.position_ms &&
    a.playing === b.playing &&
    a.rate === b.rate
  );
}

export const useIslandStore = create<IslandStore>((set, get) => ({
  snapshot: idleSnapshot,
  receivedAt: Date.now(),
  // Snapshot juga dikirim ulang karena hover, klik, atau mode berganti dengan posisi media lama.
  // Mengatur ulang receivedAt saat itu akan memundurkan progress dan lirik; jadi hanya diatur ulang
  // bila posisi, status putar, atau kecepatan benar-benar berubah.
  setSnapshot: (snapshot) =>
    set(
      sameTimeline(get().snapshot.media, snapshot.media)
        ? { snapshot }
        : { snapshot, receivedAt: Date.now() },
    ),
}));
