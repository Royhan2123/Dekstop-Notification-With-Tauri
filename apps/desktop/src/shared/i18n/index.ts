import id from "./id.json";

export type MessageKey = keyof typeof id;

// shortcut: hanya bahasa Indonesia, tambah en.json + pilihan bahasa saat Setelan dibuat (Fase 3).
export function t(key: MessageKey): string {
  return id[key];
}
