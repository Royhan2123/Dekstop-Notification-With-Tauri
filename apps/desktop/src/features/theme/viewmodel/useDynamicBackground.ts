import { useEffect, useState } from "react";
import { commands } from "../../../shared/ipc";
import { accentOfImage } from "../../../shared/theme/photo";
import { imageScene, timeScene } from "../../../shared/theme/scenes";
import { useIslandStore } from "../../island/store";
import { useSettingsStore } from "../../settings/store";
import { useDynamicStore } from "../dynamicStore";

const CLOCK_MS = 60_000;
const WALLPAPER_MS = 5 * 60_000;

/**
 * Latar pulau yang berubah sendiri menurut `background_mode`: sampul lagu yang diputar, jam, atau
 * wallpaper Windows. Hasilnya diterapkan `useSettingsSync`; tanpa sumbernya (mis. tidak ada lagu)
 * pulau memakai latar dari setelan.
 */
export function useDynamicBackground() {
  const mode = useSettingsStore((s) => s.settings?.background_mode ?? "static");
  const cover = useIslandStore((s) => s.snapshot.media?.thumbnail ?? null);
  const setScene = useDynamicStore((s) => s.setScene);
  const [hour, setHour] = useState(() => new Date().getHours());

  useEffect(() => {
    if (mode !== "time") return;
    const timer = setInterval(() => setHour(new Date().getHours()), CLOCK_MS);
    return () => clearInterval(timer);
  }, [mode]);

  useEffect(() => {
    if (mode === "time") setScene(timeScene(hour));
  }, [mode, hour, setScene]);

  useEffect(() => {
    if (mode !== "album") return;
    if (!cover) {
      setScene(null);
      return;
    }
    let alive = true;
    void accentOfImage(cover).then((accent) => alive && setScene(imageScene(cover, accent)));
    return () => {
      alive = false;
    };
  }, [mode, cover, setScene]);

  useEffect(() => {
    if (mode !== "wallpaper") return;
    let alive = true;
    const load = () =>
      void commands.wallpaperCurrent().then(async (r) => {
        if (!alive) return;
        if (r.status !== "ok") return setScene(null);
        const accent = await accentOfImage(r.data);
        if (alive) setScene(imageScene(r.data, accent));
      });
    load();
    const timer = setInterval(load, WALLPAPER_MS);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [mode, setScene]);

  useEffect(() => {
    if (mode === "static") setScene(null);
  }, [mode, setScene]);
}
