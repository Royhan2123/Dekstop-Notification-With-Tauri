import { useEffect, useRef, useState } from "react";
import { useReducedMotion } from "motion/react";
import { useSettingsStore } from "../../settings/store";
import { useIslandStore } from "../../island/store";

const POKE_MS = 700;

/** Ekspresi dari backend + reaksi "geli" saat diklik. `calm` (idle atau gerak dikurangi) mematikan semua animasi tak berujung: animasi terus-menerus memakan CPU renderer. */
export function useLumiVM() {
  const mood = useIslandStore((s) => s.snapshot.mood);
  const idle = useIslandStore((s) => s.snapshot.mode === "idle");
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const [poked, setPoked] = useState(false);
  const timer = useRef<number>(0);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  return {
    mood,
    poked,
    calm: idle || systemReduce === true || settingReduce,
    poke: () => {
      setPoked(true);
      window.clearTimeout(timer.current);
      timer.current = window.setTimeout(() => setPoked(false), POKE_MS);
    },
  };
}
