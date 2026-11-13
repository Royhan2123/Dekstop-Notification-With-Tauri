import { useEffect, useState } from "react";
import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import type { AiProvider } from "../../../bindings";
import { JarvisBrain } from "./JarvisBrain";
import { useSettingsVM } from "../viewmodel/useSettingsVM";

const PROVIDERS: AiProvider[] = [
  "claude_api",
  "gemini",
  "deep_seek",
  "grok",
  "open_ai",
  "open_router",
  "groq",
  "ollama",
  "claude_cli",
  "jarvis",
];
/** Penyedia yang tidak memakai API Key. */
const NO_KEY: AiProvider[] = ["ollama", "claude_cli", "jarvis"];

type Check = { state: "idle" | "testing" | "ok" | "error"; text: string };

/** Pilihan otak AI + API Key; dipakai di Setelan dan Onboarding. Kunci yang ditempel langsung disimpan dan diuji. */
export function AiBrainPicker() {
  const t = useT();
  const vm = useSettingsVM();
  const [key, setKey] = useState("");
  const [check, setCheck] = useState<Check>({ state: "idle", text: "" });
  const [models, setModels] = useState<string[]>([]);
  const provider = vm.settings?.ai_provider;
  const needsKey = provider !== undefined && !NO_KEY.includes(provider);
  const keyReady = !needsKey || vm.apiKeySaved === true;

  // Daftar model ikut penyedia dan kunci: begitu kunci tersimpan, pilihan model terisi dari penyedianya.
  useEffect(() => {
    setModels([]);
    if (!provider || !keyReady) return;
    let alive = true;
    void commands.aiModels().then((r) => {
      if (alive && r.status === "ok") setModels(r.data);
    });
    return () => {
      alive = false;
    };
  }, [provider, keyReady]);

  if (!vm.settings) return null;
  const { settings } = vm;

  async function test() {
    setCheck({ state: "testing", text: "" });
    const r = await commands.aiTest();
    setCheck(
      r.status === "ok"
        ? { state: "ok", text: `${t("settings.ai.connected")}${r.data ? `: ${r.data}` : ""}` }
        : { state: "error", text: r.error },
    );
  }

  async function saveAndTest(value: string) {
    if (!value.trim() || !(await vm.saveApiKey(value))) return;
    setKey("");
    await test();
  }

  return (
    <>
      <fieldset className="field">
        <legend>{t("settings.ai")}</legend>
        <label className="column">
          {t("settings.ai.provider")}
          <select
            value={settings.ai_provider}
            onChange={(e) => {
              setCheck({ state: "idle", text: "" });
              void vm.update({ ai_provider: e.currentTarget.value as AiProvider });
            }}
          >
            {PROVIDERS.map((p) => (
              <option key={p} value={p}>
                {t(`settings.ai.${p}`)}
              </option>
            ))}
          </select>
        </label>
        <label className="column">
          {t("settings.ai_model")}
          {models.length > 0 ? (
            <select
              value={settings.ai_model}
              onChange={(e) => void vm.update({ ai_model: e.currentTarget.value })}
            >
              <option value="">{t("settings.ai.model_auto")}</option>
              {settings.ai_model && !models.includes(settings.ai_model) && (
                <option value={settings.ai_model}>{settings.ai_model}</option>
              )}
              {models.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          ) : (
            <input
              key={settings.ai_model}
              defaultValue={settings.ai_model}
              maxLength={100}
              onBlur={(e) => void vm.update({ ai_model: e.currentTarget.value })}
            />
          )}
        </label>
        {needsKey && (
          <div className="column">
            <span>
              {t("settings.api_key")}:{" "}
              <strong>
                {vm.apiKeySaved ? t("settings.api_key.saved") : t("settings.api_key.missing")}
              </strong>
            </span>
            <div className="row">
              <input
                type="password"
                autoComplete="off"
                value={key}
                placeholder={t("settings.api_key.placeholder")}
                onChange={(e) => setKey(e.currentTarget.value)}
                onPaste={(e) => {
                  // Tempel = langsung simpan dan uji, tanpa menekan tombol.
                  e.preventDefault();
                  void saveAndTest(e.clipboardData.getData("text"));
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter") void saveAndTest(key);
                }}
              />
              <button type="button" disabled={!key.trim()} onClick={() => void saveAndTest(key)}>
                {t("settings.api_key.save")}
              </button>
              {vm.apiKeySaved && (
                <button
                  type="button"
                  onClick={() => {
                    setCheck({ state: "idle", text: "" });
                    void vm.removeApiKey();
                  }}
                >
                  {t("settings.api_key.remove")}
                </button>
              )}
            </div>
          </div>
        )}
        {(!needsKey || vm.apiKeySaved) && (
          <div className="row">
            <button type="button" disabled={check.state === "testing"} onClick={() => void test()}>
              {t("settings.ai.test")}
            </button>
            {check.state !== "idle" && (
              <span role="status">
                {check.state === "testing" ? "…" : check.state === "ok" ? "✓ " : "✗ "}
                {check.text}
              </span>
            )}
          </div>
        )}
      </fieldset>
      {provider === "jarvis" && <JarvisBrain />}
    </>
  );
}
