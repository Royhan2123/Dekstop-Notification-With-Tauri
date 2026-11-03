import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useReducedMotion } from "motion/react";
import { useSettingsStore } from "../../settings/store";
import { useIslandStore } from "../../island/store";

const POKE_MS = 700;
const BLINK_MS = 140;
/** Jarak kursor (piksel) yang membuat pupil mentok di tepi mata. */
const LOOK_RANGE = 240;

const clamp = (n: number) => Math.max(-1, Math.min(1, n));

/**
 * Ekspresi dari backend + reaksi "geli" saat diklik. `calm` mematikan animasi tak berujung
 * (idle atau gerak dikurangi: animasi terus-menerus memakan CPU renderer). Kedip dan mata
 * yang mengikuti kursor digerakkan lewat timer/event, jadi murah dan tetap hidup saat idle.
 */
export function useLumiVM() {
  const mood = useIslandStore((s) => s.snapshot.mood);
  const idle = useIslandStore((s) => s.snapshot.mode === "idle");
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const reduce = systemReduce === true || settingReduce;
  const [poked, setPoked] = useState(false);
  const timer = useRef<number>(0);
  const ref = useRef<SVGSVGElement>(null);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  // Kedip acak tiap 2-5 detik.
  useEffect(() => {
    if (reduce) return;
    let t = 0;
    const schedule = () => {
      t = window.setTimeout(
        () => {
          ref.current?.classList.add("blinking");
          t = window.setTimeout(() => {
            ref.current?.classList.remove("blinking");
            schedule();
          }, BLINK_MS);
        },
        2000 + Math.random() * 3000,
      );
    };
    schedule();
    return () => window.clearTimeout(t);
  }, [reduce]);

  // Pupil mengikuti kursor; posisi kursor datang dari backend (jendela pulau klik-tembus).
  useEffect(() => {
    if (reduce) return;
    const stop = listen<[number, number]>("cursor", (e) => {
      const el = ref.current;
      if (!el) return;
      const r = el.getBoundingClientRect();
      const [x, y] = e.payload;
      el.style.setProperty("--lx", String(clamp((x - (r.left + r.width / 2)) / LOOK_RANGE)));
      el.style.setProperty("--ly", String(clamp((y - (r.top + r.height / 2)) / LOOK_RANGE)));
    });
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, [reduce]);

  return {
    ref,
    mood,
    poked,
    calm: idle || reduce,
    poke: () => {
      setPoked(true);
      window.clearTimeout(timer.current);
      timer.current = window.setTimeout(() => setPoked(false), POKE_MS);
    },
  };
}
