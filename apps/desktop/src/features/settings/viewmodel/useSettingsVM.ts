import { useEffect } from "react";
import { commands, type Settings } from "../../../shared/ipc";
import { useSettingsStore } from "../store";

export function useSettingsVM() {
  const { settings, apiKeySaved, error, setSettings, setApiKeySaved, setError } =
    useSettingsStore();
  const provider = settings?.ai_provider;

  // Kunci disimpan per penyedia, jadi statusnya dibaca ulang tiap penyedia berganti.
  useEffect(() => {
    if (!provider) return;
    setApiKeySaved(false);
    let alive = true;
    void commands.apiKeyStatus(provider).then((r) => {
      if (alive && r.status === "ok") setApiKeySaved(r.data);
    });
    return () => {
      alive = false;
    };
  }, [provider, setApiKeySaved]);

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
    if (!provider) return false;
    const r = await commands.apiKeySet(provider, key);
    if (r.status === "ok") {
      setApiKeySaved(true);
      setError(null);
    } else {
      setError(r.error);
    }
    return r.status === "ok";
  }

  async function removeApiKey() {
    if (!provider) return;
    const r = await commands.apiKeyClear(provider);
    if (r.status === "ok") setApiKeySaved(false);
    else setError(r.error);
  }

  async function finishOnboarding() {
    await update({ onboarded: true });
    await commands.closeSettings();
  }

  return {
    settings,
    apiKeySaved,
    error,
    fail: setError,
    update,
    saveApiKey,
    removeApiKey,
    finishOnboarding,
  };
}
