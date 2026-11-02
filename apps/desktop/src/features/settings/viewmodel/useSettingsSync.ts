import { useEffect } from "react";
import { useLanguageStore } from "../../../shared/i18n";
import { subscribeSettings } from "../../../shared/ipc";
import { useSettingsStore } from "../store";

/** Menyinkronkan setelan dari backend ke store dan bahasa; dipanggil di akar tiap jendela. */
export function useSettingsSync() {
  const setSettings = useSettingsStore((s) => s.setSettings);
  const setLanguage = useLanguageStore((s) => s.setLanguage);

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
