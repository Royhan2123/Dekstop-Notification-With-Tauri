/** Latar siap pakai: dasar gelap dengan beberapa pendar warna, digambar ke gambar kecil (JPEG) seperti foto latar. */
interface Blob {
  x: number;
  y: number;
  /** Jari-jari relatif terhadap lebar gambar. */
  r: number;
  color: string;
}

export interface Preset {
  id: string;
  name: string;
  accent: string;
  base: string;
  blobs: Blob[];
}

export const PRESETS: Preset[] = [
  {
    id: "aurora",
    name: "Aurora",
    accent: "#2fcf9c",
    base: "#06141c",
    blobs: [
      { x: 0.12, y: 0.15, r: 0.6, color: "#22d3a6" },
      { x: 0.88, y: 0.85, r: 0.6, color: "#6d5df6" },
      { x: 0.55, y: 0.05, r: 0.4, color: "#38a3f5" },
    ],
  },
  {
    id: "sunset",
    name: "Sunset",
    accent: "#ff8a4c",
    base: "#1c0c17",
    blobs: [
      { x: 0.1, y: 0.95, r: 0.65, color: "#ff6b4a" },
      { x: 0.92, y: 0.1, r: 0.6, color: "#c026d3" },
      { x: 0.55, y: 0.55, r: 0.3, color: "#ff9a6c" },
    ],
  },
  {
    id: "ocean",
    name: "Ocean",
    accent: "#38a3f5",
    base: "#051223",
    blobs: [
      { x: 0.18, y: 0.2, r: 0.6, color: "#0ea5e9" },
      { x: 0.9, y: 0.9, r: 0.65, color: "#1d4ed8" },
      { x: 0.6, y: 0.55, r: 0.3, color: "#22d3ee" },
    ],
  },
  {
    id: "nebula",
    name: "Nebula",
    accent: "#a78bfa",
    base: "#0d0720",
    blobs: [
      { x: 0.2, y: 0.85, r: 0.62, color: "#a855f7" },
      { x: 0.85, y: 0.15, r: 0.6, color: "#ec4899" },
      { x: 0.5, y: 0.5, r: 0.3, color: "#6366f1" },
    ],
  },
  {
    id: "sakura",
    name: "Sakura",
    accent: "#f472b6",
    base: "#1c0b16",
    blobs: [
      { x: 0.12, y: 0.12, r: 0.55, color: "#f472b6" },
      { x: 0.88, y: 0.88, r: 0.6, color: "#fb7185" },
      { x: 0.62, y: 0.3, r: 0.3, color: "#fda4af" },
    ],
  },
  {
    id: "forest",
    name: "Forest",
    accent: "#4ade80",
    base: "#05140c",
    blobs: [
      { x: 0.1, y: 0.5, r: 0.62, color: "#16a34a" },
      { x: 0.92, y: 0.5, r: 0.55, color: "#0f766e" },
      { x: 0.5, y: 0.08, r: 0.3, color: "#a3e635" },
    ],
  },
  {
    id: "graphite",
    name: "Graphite",
    accent: "#9db4ff",
    base: "#0c0d12",
    blobs: [
      { x: 0.2, y: 0.2, r: 0.6, color: "#475569" },
      { x: 0.92, y: 0.9, r: 0.6, color: "#334155" },
      { x: 0.6, y: 0.4, r: 0.3, color: "#94a3b8" },
    ],
  },
  {
    id: "gold",
    name: "Gold",
    accent: "#e6c619",
    base: "#16110a",
    blobs: [
      { x: 0.2, y: 0.85, r: 0.62, color: "#f59e0b" },
      { x: 0.82, y: 0.15, r: 0.55, color: "#eab308" },
      { x: 0.5, y: 0.5, r: 0.3, color: "#fde047" },
    ],
  },
];

const W = 480;
const H = 160;
const cache = new Map<string, string>();

/** Gambar preset ke data URL JPEG (±10 KB, lolos saringan latar di backend); hasilnya disimpan agar tidak digambar ulang. */
export function renderPreset(p: Preset): string {
  const hit = cache.get(p.id);
  if (hit) return hit;
  const canvas = document.createElement("canvas");
  canvas.width = W;
  canvas.height = H;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("canvas");
  ctx.fillStyle = p.base;
  ctx.fillRect(0, 0, W, H);
  for (const b of p.blobs) {
    const g = ctx.createRadialGradient(b.x * W, b.y * H, 0, b.x * W, b.y * H, b.r * W);
    g.addColorStop(0, `${b.color}cc`);
    g.addColorStop(1, `${b.color}00`);
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, W, H);
  }
  const url = canvas.toDataURL("image/jpeg", 0.82);
  cache.set(p.id, url);
  return url;
}
