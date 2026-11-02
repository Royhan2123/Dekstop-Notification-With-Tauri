import { useEffect } from "react";
import { commands, type Settings } from "../../../shared/ipc";
import { useSettingsStore } from "../store";

export function useSettingsVM() {
  const { settings, apiKeySaved, error, setSettings, setApiKeySaved, setError } =
    useSettingsStore();

  useEffect(() => {
    void commands.apiKeyStatus().then((r) => {
      if (r.status === "ok") setApiKeySaved(r.data);
    });
  }, [setApiKeySaved]);

  async function update(patch: Partial<Settings>) {
    if (!settings) return;
    const r = await commands.settingsUpdate({ ...settings, ...patch });
    if (r.status === "ok") {
      setSettings(r.data);
      setError(null);
    } else {
      setError(r.error);
    }
  }

  async function saveApiKey(key: string): Promise<boolean> {
    const r = await commands.apiKeySet(key);
    if (r.status === "ok") {
      setApiKeySaved(true);
      setError(null);
    } else {
      setError(r.error);
    }
    return r.status === "ok";
  }

  async function removeApiKey() {
    const r = await commands.apiKeyClear();
    if (r.status === "ok") setApiKeySaved(false);
    else setError(r.error);
  }

  async function finishOnboarding() {
    await update({ onboarded: true });
    await commands.closeSettings();
  }

  return { settings, apiKeySaved, error, update, saveApiKey, removeApiKey, finishOnboarding };
}
