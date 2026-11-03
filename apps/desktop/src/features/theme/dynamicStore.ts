import { create } from "zustand";
import type { Scene } from "../../shared/theme/scenes";

interface DynamicStore {
  /** Suasana dari sampul album, jam, atau wallpaper; `null` memakai latar dan aksen dari setelan. */
  scene: Scene | null;
  setScene: (scene: Scene | null) => void;
}

export const useDynamicStore = create<DynamicStore>((set) => ({
  scene: null,
  setScene: (scene) => set({ scene }),
}));
