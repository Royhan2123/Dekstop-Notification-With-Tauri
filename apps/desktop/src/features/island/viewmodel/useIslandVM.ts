import { useEffect } from "react";
import { commands, subscribeIsland, type IslandSnapshot } from "../../../shared/ipc";
import { useIslandStore } from "../store";

export interface Shape {
  width: number;
  height: number;
  radius: number;
  opacity: number;
}

/** Ukuran pulau (piksel logis) untuk tiap mode; `hovered` hanya melebarkan sedikit saat idle. */
export function shapeFor(s: IslandSnapshot): Shape {
  switch (s.mode) {
    case "hidden":
      return { width: 0, height: 0, radius: 0, opacity: 0 };
    case "idle":
      return s.hovered
        ? { width: 150, height: 36, radius: 18, opacity: 1 }
        : { width: 120, height: 32, radius: 16, opacity: 1 };
    case "compact":
      return { width: 240, height: 36, radius: 18, opacity: 1 };
    case "expanded":
      // Kartu review berisi temuan butuh ruang untuk daftar dan tombol keputusan.
      if (s.review && s.review.findings.length > 0) {
        return { width: 480, height: 250, radius: 28, opacity: 1 };
      }
      return { width: 420, height: 140, radius: 28, opacity: 1 };
  }
}

export function useIslandVM() {
  const snapshot = useIslandStore((s) => s.snapshot);
  const setSnapshot = useIslandStore((s) => s.setSnapshot);

  useEffect(() => {
    const stop = subscribeIsland(setSnapshot);
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, [setSnapshot]);

  return {
    mode: snapshot.mode,
    shape: shapeFor(snapshot),
    content: snapshot.content,
    hasFindings: (snapshot.review?.findings.length ?? 0) > 0,
    onHover: (hovered: boolean) => void commands.islandHover(hovered),
    onClick: () => void commands.islandClick(),
    onDismiss: () => void commands.islandDismiss(),
    // shortcut: klik kanan langsung membuka Setelan; menu cepat (Fokus, Jeda notifikasi) menyusul.
    onOpenSettings: () => void commands.openSettings(),
    reportHitbox: (r: DOMRect) =>
      void commands.islandHitbox({ x: r.x, y: r.y, width: r.width, height: r.height }),
  };
}
