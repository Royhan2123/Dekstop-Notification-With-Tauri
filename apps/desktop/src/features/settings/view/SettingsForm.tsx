import { useT } from "../../../shared/i18n";
import type { Language } from "../../../bindings";
import { useSettingsVM } from "../viewmodel/useSettingsVM";
import { QuickAskHint } from "./QuickAskHint";
import { ClipboardSettings } from "../../clipboard/view/ClipboardSettings";
import { TerminalNotifyHint } from "./TerminalNotifyHint";
import { PomodoroSettings } from "../../pomodoro/view/PomodoroSettings";
import { LinkSettings } from "../../link/view/LinkSettings";
import { HealthSettings } from "../../health/view/HealthSettings";
import { AiBrainPicker } from "./AiBrainPicker";
import { ReviewSettings } from "./ReviewSettings";
import { ThemeSettings } from "../../theme/view/ThemeSettings";
import { CalendarSettings } from "../../calendar/view/CalendarSettings";
import { GithubSettings } from "../../github/view/GithubSettings";

const LANGUAGES: Language[] = ["id", "en"];

export function SettingsForm() {
  const t = useT();
  const vm = useSettingsVM();
  if (!vm.settings) return null;
  const { settings } = vm;

  return (
    <div className="settings">
      <h1>{t("settings.title")}</h1>

      <fieldset className="field">
        <legend>{t("settings.general")}</legend>
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

        <label className="row">
          <input
            type="checkbox"
            checked={settings.hide_on_fullscreen}
            onChange={(e) => void vm.update({ hide_on_fullscreen: e.currentTarget.checked })}
          />
          {t("settings.hide_on_fullscreen")}
        </label>

        <label className="row">
          <input
            type="checkbox"
            checked={settings.lyrics_enabled}
            onChange={(e) => void vm.update({ lyrics_enabled: e.currentTarget.checked })}
          />
          {t("settings.lyrics")}
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
      </fieldset>

      <ThemeSettings />
      <CalendarSettings />
      <GithubSettings />

      <AiBrainPicker />

      <ReviewSettings />

      <PomodoroSettings />

      <QuickAskHint />

      <TerminalNotifyHint />

      <ClipboardSettings />

      <LinkSettings />

      <HealthSettings />

      {vm.error && (
        <p role="alert" className="error">
          {t("settings.error", vm.error)}
        </p>
      )}
    </div>
  );
}
