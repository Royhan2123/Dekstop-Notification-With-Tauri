import { useEffect, useState } from "react";

const KEY = "isla.wellness";
/** Target gelas air per hari dan lama istirahat mata (aturan 20-20-20: 20 detik melihat jauh). */
export const WATER_GOAL = 8;
export const EYE_SECS = 20;
const WATER_MAX = 24;
const EYES_MAX = 99;
/** Lama pesan "selesai" tampil setelah istirahat mata. */
const DONE_MS = 2200;

/** Catatan harian yang disimpan di perangkat; kembali nol saat tanggal berganti. */
export interface Wellness {
  date: string;
  water: number;
  eyes: number;
}

const clamp = (n: unknown, max: number) =>
  typeof n === "number" && Number.isInteger(n) ? Math.min(max, Math.max(0, n)) : 0;

/** Tanggal lokal YYYY-MM-DD. */
export const todayKey = () => new Date().toLocaleDateString("sv");

/** Baca catatan tersimpan; isi rusak atau dari hari lain dianggap kosong. */
export function loadWellness(raw: string | null, today: string): Wellness {
  try {
    const j: unknown = JSON.parse(raw ?? "null");
    if (j && typeof j === "object" && (j as Wellness).date === today) {
      const w = j as Wellness;
      return { date: today, water: clamp(w.water, WATER_MAX), eyes: clamp(w.eyes, EYES_MAX) };
    }
  } catch {
    // isi rusak: mulai dari nol
  }
  return { date: today, water: 0, eyes: 0 };
}

function read(): Wellness {
  try {
    return loadWellness(localStorage.getItem(KEY), todayKey());
  } catch {
    return loadWellness(null, todayKey());
  }
}

function write(w: Wellness) {
  try {
    localStorage.setItem(KEY, JSON.stringify(w));
  } catch {
    // penyimpanan penuh atau dimatikan: hitungan tetap tampil selama panel terbuka
  }
}

/** Air minum dan istirahat mata hari ini, plus sesi istirahat mata berhitung mundur. */
export function useWellnessVM() {
  const [state, setState] = useState(read);
  const [eyeLeft, setEyeLeft] = useState<number | null>(null);
  const [eyeDone, setEyeDone] = useState(false);

  const bump = (field: "water" | "eyes") => {
    const cur = read();
    const max = field === "water" ? WATER_MAX : EYES_MAX;
    const next = { ...cur, [field]: Math.min(max, cur[field] + 1) };
    write(next);
    setState(next);
  };

  useEffect(() => {
    if (eyeLeft === null || eyeDone) return;
    if (eyeLeft <= 0) {
      bump("eyes");
      setEyeDone(true);
      return;
    }
    const id = window.setTimeout(() => setEyeLeft(eyeLeft - 1), 1000);
    return () => window.clearTimeout(id);
  }, [eyeLeft, eyeDone]);

  useEffect(() => {
    if (!eyeDone) return;
    const id = window.setTimeout(() => {
      setEyeLeft(null);
      setEyeDone(false);
    }, DONE_MS);
    return () => window.clearTimeout(id);
  }, [eyeDone]);

  return {
    water: state.water,
    eyes: state.eyes,
    /** Detik tersisa; `null` bila tidak sedang istirahat. */
    eyeLeft,
    eyeDone,
    addWater: () => bump("water"),
    startEye: () => setEyeLeft(EYE_SECS),
    cancelEye: () => {
      setEyeLeft(null);
      setEyeDone(false);
    },
  };
}
