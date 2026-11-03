import { useEffect } from "react";
import { useLanguageStore } from "../../../shared/i18n";
import { subscribeSettings } from "../../../shared/ipc";
import { accentOf, paletteFor } from "../../../shared/theme/themes";
import { useSettingsStore } from "../store";

/** Menyinkronkan setelan dari backend ke store dan bahasa; dipanggil di akar tiap jendela. */
export function useSettingsSync() {
  const setSettings = useSettingsStore((s) => s.setSettings);
  const setLanguage = useLanguageStore((s) => s.setLanguage);
  const accent = useSettingsStore((s) =>
    s.settings ? accentOf(s.settings.theme, s.settings.custom_accent) : null,
  );

  const background = useSettingsStore((s) => s.settings?.background ?? "");

  useEffect(() => {
    const root = document.documentElement.style;
    if (background) {
      root.setProperty(
        "--island-bg",
        `linear-gradient(rgba(0,0,0,.62),rgba(0,0,0,.62)), url("${background}")`,
      );
    } else root.removeProperty("--island-bg");
  }, [background]);

  useEffect(() => {
    if (!accent) return;
    for (const [k, v] of Object.entries(paletteFor(accent))) {
      document.documentElement.style.setProperty(k, v);
    }
  }, [accent]);

  useEffect(() => {
    const stop = subscribeSettings((s) => {
      setSettings(s);
      setLanguage(s.language);
    });
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, [setSettings, setLanguage]);
}
