/** Tema bawaan: satu warna aksen; seluruh palet pulau dan Mino diturunkan darinya. */
export const THEMES = [
  { id: "mint", accent: "#2fcf9c" },
  { id: "ocean", accent: "#38a3f5" },
  { id: "sunset", accent: "#ff8a4c" },
  { id: "sakura", accent: "#f472b6" },
  { id: "grape", accent: "#a78bfa" },
  { id: "lemon", accent: "#e6c619" },
] as const;

function hslOf(hex: string): [number, number, number] {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  const d = max - min;
  if (d === 0) return [0, 0, l * 100];
  const s = d / (1 - Math.abs(2 * l - 1));
  const h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [(h * 60 + 360) % 360, s * 100, l * 100];
}

const hsl = (h: number, s: number, l: number) =>
  `hsl(${Math.round(h)} ${Math.round(s)}% ${Math.round(l)}%)`;

/** Variabel CSS untuk satu aksen; dipasang di `:root` tiap jendela. */
export function paletteFor(accent: string): Record<string, string> {
  const [h, s0] = hslOf(accent);
  const s = Math.max(s0, 45);
  return {
    "--accent": accent,
    "--accent-ink": hsl(h, 70, 8),
    "--m-hi": hsl(h, 100, 96),
    "--m-mid": hsl(h, s, 78),
    "--m-lo": hsl(h, s, 42),
    "--m-visor": hsl(h, 60, 10),
    "--m-eye": hsl(h, 100, 72),
    "--m-soft": hsl(h, s, 68),
  };
}

export function accentOf(theme: string, custom: string): string {
  if (/^#[0-9a-fA-F]{6}$/.test(custom)) return custom;
  return (THEMES.find((t) => t.id === theme) ?? THEMES[0]).accent;
}

export const MASCOTS = ["mino", "kiko", "puff"] as const;

function hex(h: number, s: number, l: number): string {
  const a = (s / 100) * Math.min(l / 100, 1 - l / 100);
  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    const c = l / 100 - a * Math.max(-1, Math.min(k - 3, 9 - k, 1));
    return Math.round(c * 255)
      .toString(16)
      .padStart(2, "0");
  };
  return `#${f(0)}${f(8)}${f(4)}`;
}

/** Warna aksen dari piksel RGBA sebuah foto: rata-rata berbobot kejenuhan, lalu dibuat cukup terang dan hidup. */
export function accentFromPixels(data: Uint8ClampedArray): string {
  let r = 0;
  let g = 0;
  let b = 0;
  let w = 0;
  for (let i = 0; i + 3 < data.length; i += 4) {
    const max = Math.max(data[i], data[i + 1], data[i + 2]);
    const min = Math.min(data[i], data[i + 1], data[i + 2]);
    const weight = (max - min) / 255 + 0.02;
    r += data[i] * weight;
    g += data[i + 1] * weight;
    b += data[i + 2] * weight;
    w += weight;
  }
  if (w === 0) return THEMES[0].accent;
  const avg = `#${[r, g, b]
    .map((v) =>
      Math.round(v / w)
        .toString(16)
        .padStart(2, "0"),
    )
    .join("")}`;
  const [h, s] = hslOf(avg);
  return hex(h, Math.max(s, 60), 58);
}
