import { useEffect, useState } from "react";

const KEY = "isla.wellness";
/** Target gelas air per hari dan lama istirahat mata (aturan 20-20-20: 20 detik melihat jauh). */
export const WATER_GOAL = 8;
export const EYE_SECS = 20;
/** Target harian untuk skor: istirahat mata dan sesi peregangan. */
export const EYE_GOAL = 6;
export const STRETCH_GOAL = 3;
const MAX = { water: 24, eyes: 99, stretch: 99, breath: 99 } as const;
/** Lama pesan "selesai" tampil setelah istirahat mata. */
const DONE_MS = 2200;

/** Catatan harian yang disimpan di perangkat; kembali nol saat tanggal berganti. */
export interface Wellness {
  date: string;
  water: number;
  eyes: number;
  stretch: number;
  breath: number;
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
      return {
        date: today,
        water: clamp(w.water, MAX.water),
        eyes: clamp(w.eyes, MAX.eyes),
        stretch: clamp(w.stretch, MAX.stretch),
        breath: clamp(w.breath, MAX.breath),
      };
    }
  } catch {
    // isi rusak: mulai dari nol
  }
  return { date: today, water: 0, eyes: 0, stretch: 0, breath: 0 };
}

/** Catatan hari ini dari penyimpanan (dipakai juga oleh pengingat). */
export function readWellness(): Wellness {
  try {
    return loadWellness(localStorage.getItem(KEY), todayKey());
  } catch {
    return loadWellness(null, todayKey());
  }
}

const read = readWellness;

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

  const bump = (field: keyof typeof MAX) => {
    const cur = read();
    const next = { ...cur, [field]: Math.min(MAX[field], cur[field] + 1) };
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
    stretch: state.stretch,
    breath: state.breath,
    /** Detik tersisa; `null` bila tidak sedang istirahat. */
    eyeLeft,
    eyeDone,
    addWater: () => bump("water"),
    addStretch: () => bump("stretch"),
    addBreath: () => bump("breath"),
    startEye: () => setEyeLeft(EYE_SECS),
    cancelEye: () => {
      setEyeLeft(null);
      setEyeDone(false);
    },
  };
}
