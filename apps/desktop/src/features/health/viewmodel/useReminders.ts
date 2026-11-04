import { useEffect, useRef } from "react";
import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { readWellness } from "./useWellnessVM";

export const REMINDER_MIN = { water: 60, eyes: 20, stretch: 50 } as const;
export type ReminderKind = keyof typeof REMINDER_MIN;
export type ReminderPrefs = Record<ReminderKind, boolean>;

export const PREFS_KEY = "isla.reminders";
const CHECK_MS = 30_000;
const KINDS = Object.keys(REMINDER_MIN) as ReminderKind[];

/** Setelan pengingat tersimpan; isi rusak atau kosong berarti semua menyala. */
export function loadPrefs(raw: string | null): ReminderPrefs {
  const on = { water: true, eyes: true, stretch: true };
  try {
    const j: unknown = JSON.parse(raw ?? "null");
    if (j && typeof j === "object") {
      for (const k of KINDS) if ((j as Record<string, unknown>)[k] === false) on[k] = false;
    }
  } catch {
    // isi rusak: semua menyala
  }
  return on;
}

export function savePrefs(p: ReminderPrefs) {
  try {
    localStorage.setItem(PREFS_KEY, JSON.stringify(p));
  } catch {
    // penyimpanan dimatikan: berlaku sampai app ditutup
  }
}

/** Jenis pengingat yang sudah lewat jatuh temponya dan menyala. */
export function dueKinds(
  nowMs: number,
  lastMs: Record<ReminderKind, number>,
  prefs: ReminderPrefs,
): ReminderKind[] {
  return KINDS.filter((k) => prefs[k] && nowMs - lastMs[k] >= REMINDER_MIN[k] * 60_000);
}

/**
 * Pengingat sehat otomatis: air, mata, peregangan. Dipasang sekali di pulau; tiap jatuh tempo
 * muncul pemberitahuan singkat (diam saat Jangan Ganggu). Mencatat kebiasaan di panel menunda pengingatnya.
 */
export function useReminders() {
  const t = useT();
  const tRef = useRef(t);
  tRef.current = t;

  useEffect(() => {
    const start = Date.now();
    const last = { water: start, eyes: start, stretch: start };
    const w0 = readWellness();
    const seen = { water: w0.water, eyes: w0.eyes, stretch: w0.stretch };
    const id = window.setInterval(() => {
      const now = Date.now();
      const w = readWellness();
      for (const k of KINDS) {
        if (w[k] > seen[k]) last[k] = now;
        seen[k] = w[k];
      }
      const [first] = dueKinds(now, last, loadPrefs(localStorage.getItem(PREFS_KEY)));
      if (!first) return;
      last[first] = now;
      void commands.islandNotice(tRef.current(`health.remind.${first}`));
    }, CHECK_MS);
    return () => window.clearInterval(id);
  }, []);
}
