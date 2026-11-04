import { useEffect, useState } from "react";

/** Angka yang naik halus dari nilai sebelumnya ke `target`; langsung bila animasi dikurangi. */
export function useCountUp(target: number, instant: boolean, ms = 800): number {
  const [value, setValue] = useState(instant ? target : 0);

  useEffect(() => {
    if (instant) {
      setValue(target);
      return;
    }
    const from = value;
    const t0 = performance.now();
    let raf = requestAnimationFrame(function step(now) {
      const p = Math.min(1, (now - t0) / ms);
      setValue(Math.round(from + (target - from) * (1 - Math.pow(1 - p, 3))));
      if (p < 1) raf = requestAnimationFrame(step);
    });
    return () => cancelAnimationFrame(raf);
    // `value` sengaja tidak jadi dependensi: titik awal tween hanya diambil saat target berubah.
  }, [target, instant, ms]);

  return value;
}
