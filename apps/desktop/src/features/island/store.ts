import { create } from "zustand";
import type { IslandSnapshot } from "../../shared/ipc";

export const idleSnapshot: IslandSnapshot = {
  mode: "idle",
  content: null,
  media: null,
  review: null,
  hovered: false,
};

interface IslandStore {
  snapshot: IslandSnapshot;
  /** Waktu (ms) snapshot diterima; dasar untuk menghitung progress media. */
  receivedAt: number;
  setSnapshot: (s: IslandSnapshot) => void;
}

export const useIslandStore = create<IslandStore>((set) => ({
  snapshot: idleSnapshot,
  receivedAt: Date.now(),
  setSnapshot: (snapshot) => set({ snapshot, receivedAt: Date.now() }),
}));
