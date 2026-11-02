import { useState } from "react";
import { useT } from "../../../shared/i18n";
import type { AiProvider } from "../../../bindings";
import { useSettingsVM } from "../viewmodel/useSettingsVM";

const PROVIDERS: AiProvider[] = ["claude_api", "ollama", "claude_cli"];

/** Pilihan otak AI + API Key; dipakai di Setelan dan Onboarding. */
export function AiBrainPicker() {
  const t = useT();
  const vm = useSettingsVM();
  const [key, setKey] = useState("");
  if (!vm.settings) return null;
  const { settings } = vm;

  return (
    <fieldset className="field">
      <legend>{t("settings.ai")}</legend>
      {PROVIDERS.map((p) => (
        <label key={p} className="row">
          <input
            type="radio"
            name="ai_provider"
            checked={settings.ai_provider === p}
            onChange={() => void vm.update({ ai_provider: p })}
          />
          {t(`settings.ai.${p}`)}
        </label>
      ))}
      <label className="column">
        {t("settings.ai_model")}
        <input
          key={settings.ai_model}
          defaultValue={settings.ai_model}
          maxLength={100}
          onBlur={(e) => void vm.update({ ai_model: e.currentTarget.value })}
        />
      </label>
      {settings.ai_provider === "claude_api" && (
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
            />
            <button
              type="button"
              disabled={!key.trim()}
              onClick={() =>
                void vm.saveApiKey(key).then((ok) => {
                  if (ok) setKey("");
                })
              }
            >
              {t("settings.api_key.save")}
            </button>
            {vm.apiKeySaved && (
              <button type="button" onClick={() => void vm.removeApiKey()}>
                {t("settings.api_key.remove")}
              </button>
            )}
          </div>
        </div>
      )}
    </fieldset>
  );
}
