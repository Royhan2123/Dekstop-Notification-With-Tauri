import { create } from "zustand";
import type { IslandSnapshot } from "../../shared/ipc";

export const idleSnapshot: IslandSnapshot = { mode: "idle", content: null, hovered: false };

interface IslandStore {
  snapshot: IslandSnapshot;
  setSnapshot: (s: IslandSnapshot) => void;
}

export const useIslandStore = create<IslandStore>((set) => ({
  snapshot: idleSnapshot,
  setSnapshot: (snapshot) => set({ snapshot }),
}));
