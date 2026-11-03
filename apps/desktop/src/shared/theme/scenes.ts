export interface Scene {
  /** Nilai CSS `background-image` untuk pulau. */
  background: string;
  accent: string;
}

const SHADE = "linear-gradient(rgba(0,0,0,.3),rgba(0,0,0,.3))";

/** Suasana pulau menurut jam: fajar, siang, senja, malam. */
export function timeScene(hour: number): Scene {
  const scene = (from: string, to: string, accent: string): Scene => ({
    background: `${SHADE}, linear-gradient(135deg, ${from}, ${to})`,
    accent,
  });
  if (hour >= 5 && hour < 8) return scene("#3b2a5a", "#ff9a6c", "#ff9a6c");
  if (hour >= 8 && hour < 17) return scene("#0f3d5e", "#38a3f5", "#38a3f5");
  if (hour >= 17 && hour < 20) return scene("#3a1f4a", "#ff6b4a", "#ff8a4c");
  return scene("#05070f", "#1b2a5a", "#8aa4ff");
}

/** Latar dari gambar (sampul, wallpaper) dengan lapisan gelap agar teks tetap terbaca. */
export function imageScene(dataUrl: string, accent: string): Scene {
  return {
    background: `linear-gradient(rgba(0,0,0,.62),rgba(0,0,0,.62)), url("${dataUrl}")`,
    accent,
  };
}
