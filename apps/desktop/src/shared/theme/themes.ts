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
