import { create } from "zustand";
import type { ChatTurnDto } from "../../shared/ipc";

interface ChatStore {
  turns: ChatTurnDto[];
  /** Sudah dimuat dari backend; sebelum itu riwayat kosong bukan berarti belum ada chat. */
  loaded: boolean;
  setTurns: (turns: ChatTurnDto[]) => void;
}

/** Riwayat Tanya AI yang tampil di kartu; sumber kebenarannya ada di backend. */
export const useChatStore = create<ChatStore>((set) => ({
  turns: [],
  loaded: false,
  setTurns: (turns) => set({ turns, loaded: true }),
}));
