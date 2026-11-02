import { useT } from "../../../shared/i18n";
import type { Language } from "../../../bindings";
import { useSettingsVM } from "../viewmodel/useSettingsVM";
import { HealthSettings } from "../../health/view/HealthSettings";
import { AiBrainPicker } from "./AiBrainPicker";
import { ReviewSettings } from "./ReviewSettings";

const LANGUAGES: Language[] = ["id", "en"];

export function SettingsForm() {
  const t = useT();
  const vm = useSettingsVM();
  if (!vm.settings) return null;
  const { settings } = vm;

  return (
    <div className="settings">
      <h1>{t("settings.title")}</h1>

      <label className="column">
        {t("settings.language")}
        <select
          value={settings.language}
          onChange={(e) => void vm.update({ language: e.currentTarget.value as Language })}
        >
          {LANGUAGES.map((l) => (
            <option key={l} value={l}>
              {t(`settings.language.${l}`)}
            </option>
          ))}
        </select>
      </label>

      <label className="row">
        <input
          type="checkbox"
          checked={settings.reduce_motion}
          onChange={(e) => void vm.update({ reduce_motion: e.currentTarget.checked })}
        />
        {t("settings.reduce_motion")}
      </label>

      <label className="column">
        {t("settings.display_seconds")}: {settings.display_seconds}
        <input
          type="range"
          min={2}
          max={60}
          value={settings.display_seconds}
          onChange={(e) => void vm.update({ display_seconds: Number(e.currentTarget.value) })}
        />
      </label>

      <label className="row">
        <input
          type="checkbox"
          checked={settings.autostart}
          onChange={(e) => void vm.update({ autostart: e.currentTarget.checked })}
        />
        {t("settings.autostart")}
      </label>

      <AiBrainPicker />

      <ReviewSettings />

      <HealthSettings />

      {vm.error && (
        <p role="alert" className="error">
          {t("settings.error", vm.error)}
        </p>
      )}
    </div>
  );
}
