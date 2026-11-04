import { useEffect } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { commands, subscribeIsland, type IslandSnapshot } from "../../../shared/ipc";
import { useChatStore } from "../../ask/store";
import { useChatSync } from "../../ask/viewmodel/useChatSync";
import { useLyricsSync, useLyricsVM } from "../../media/viewmodel/useLyricsVM";
import { useIslandStore } from "../store";

/** Tinggi dok fitur di dasar pulau terbuka. */
export const DOCK_H = 58;

export interface Shape {
  width: number;
  height: number;
  radius: number;
  opacity: number;
}

/** Lebar kartu media yang membuka kolom lirik di sisi kanan. */
const LYRICS_W = 600;

/** Hal di luar snapshot yang ikut menentukan ukuran: ada lirik (kartu media) dan ada riwayat chat (Tanya AI). */
export interface ShapeExtras {
  lyrics?: boolean;
  chat?: boolean;
}

/** Ukuran pulau (piksel logis) untuk tiap mode; `hovered` hanya melebarkan sedikit saat idle. */
export function shapeFor(
  s: IslandSnapshot,
  { lyrics = false, chat = false }: ShapeExtras = {},
): Shape {
  switch (s.mode) {
    case "hidden":
      return { width: 0, height: 0, radius: 0, opacity: 0 };
    case "idle":
      return s.hovered
        ? { width: 112, height: 38, radius: 19, opacity: 1 }
        : { width: 84, height: 38, radius: 19, opacity: 1 };
    case "compact":
      return { width: 276, height: 40, radius: 20, opacity: 1 };
    case "expanded":
      // Kartu review berisi temuan butuh ruang untuk daftar dan tombol keputusan.
      if (s.review && s.review.findings.length > 0) {
        return { width: 520, height: 232 + DOCK_H, radius: 28, opacity: 1 };
      }
      // Rak file: kepala + satu baris per berkas (maksimal 5 terlihat, sisanya digulir).
      if (s.content?.kind === "shelf") {
        return {
          width: 440,
          height: 66 + DOCK_H + 28 * Math.min(Math.max(s.shelf.length, 1), 5),
          radius: 28,
          opacity: 1,
        };
      }
      // Quick Ask: kotak pertanyaan kecil; memanjang bila ada riwayat chat atau jawaban sedang datang.
      if (s.content?.kind === "ask") {
        const busy = s.content.detail === "done" || s.content.detail === "failed";
        return { width: 520, height: (busy || chat ? 260 : 86) + DOCK_H, radius: 28, opacity: 1 };
      }
      // Kartu media dengan ekstensi tersambung menambah baris kecepatan dan pencarian.
      if (s.extension_connected && s.content?.kind === "media") {
        return { width: lyrics ? LYRICS_W : 440, height: 186 + DOCK_H, radius: 28, opacity: 1 };
      }
      return {
        width: lyrics && s.content?.kind === "media" ? LYRICS_W : 440,
        height: 128 + DOCK_H,
        radius: 28,
        opacity: 1,
      };
  }
}

/** Hitbox klik-tembus untuk backend; fungsi tetap supaya pengamat ukuran tidak dipasang ulang tiap render. */
function reportHitbox(r: DOMRect) {
  void commands.islandHitbox({ x: r.x, y: r.y, width: r.width, height: r.height });
}

export function useIslandVM() {
  const snapshot = useIslandStore((s) => s.snapshot);
  const setSnapshot = useIslandStore((s) => s.setSnapshot);
  useLyricsSync();
  const hasLyrics = useLyricsVM().length > 0;
  useChatSync();
  const hasChat = useChatStore((s) => s.turns.length > 0);

  useEffect(() => {
    const stop = subscribeIsland(setSnapshot);
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, [setSnapshot]);

  // Berkas yang dijatuhkan ke pulau masuk rak (pulau menerima kursor hanya saat kursor di atasnya).
  useEffect(() => {
    const stop = getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type === "drop") void commands.shelfAdd(e.payload.paths);
    });
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, []);

  return {
    mode: snapshot.mode,
    shape: shapeFor(snapshot, { lyrics: hasLyrics, chat: hasChat }),
    content: snapshot.content,
    hasFindings: (snapshot.review?.findings.length ?? 0) > 0,
    onHover: (hovered: boolean) => void commands.islandHover(hovered),
    // Klik pulau kosong menampilkan ringkasan kesehatan hari ini.
    onClick: () => void (snapshot.mode === "idle" ? commands.healthPeek() : commands.islandClick()),
    onDismiss: () => void commands.islandDismiss(),
    // shortcut: klik kanan langsung membuka Setelan; menu cepat (Fokus, Jeda notifikasi) menyusul.
    onOpenSettings: () => void commands.openSettings(),
    reportHitbox,
  };
}
