import { useLayoutEffect, useRef, useState, type CSSProperties } from "react";

/** Bagian lebar kotak yang dijaga agar bagian yang sedang dinyanyikan tetap terlihat saat teks digulir. */
const FOCUS = 0.55;

/**
 * Satu baris lirik yang terwarnai sampai bagian yang sedang dinyanyikan (`fraction` 0..1).
 * Baris yang lebih lebar dari kotaknya digulir mengikuti warna, bukan dipotong titik-titik.
 */
export function KaraokeLine({ text, fraction }: { text: string; fraction: number }) {
  const box = useRef<HTMLSpanElement>(null);
  const inner = useRef<HTMLSpanElement>(null);
  const [widths, setWidths] = useState({ box: 0, text: 0 });

  useLayoutEffect(() => {
    const b = box.current;
    const t = inner.current;
    if (!b || !t) return;
    const measure = () => setWidths({ box: b.clientWidth, text: t.offsetWidth });
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(b);
    return () => observer.disconnect();
  }, [text]);

  const overflow = Math.max(0, widths.text - widths.box);
  const shift = Math.min(overflow, Math.max(0, fraction * widths.text - widths.box * FOCUS));
  const style = {
    "--p": `${(fraction * 100).toFixed(1)}%`,
    transform: `translateX(${-shift}px)`,
  } as CSSProperties;

  return (
    <span className="karaoke" ref={box}>
      <span className="karaoke-text" ref={inner} style={style}>
        {text}
      </span>
    </span>
  );
}
