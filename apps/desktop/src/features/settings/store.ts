import { create } from "zustand";
import type { Settings } from "../../shared/ipc";

interface SettingsStore {
  /** `null` sampai setelan pertama diterima dari backend. */
  settings: Settings | null;
  /** Hanya status; API Key sendiri tidak pernah sampai ke frontend. */
  apiKeySaved: boolean | null;
  error: string | null;
  setSettings: (s: Settings) => void;
  setApiKeySaved: (saved: boolean) => void;
  setError: (message: string | null) => void;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: null,
  apiKeySaved: null,
  error: null,
  setSettings: (settings) => set({ settings }),
  setApiKeySaved: (apiKeySaved) => set({ apiKeySaved }),
  setError: (error) => set({ error }),
}));
