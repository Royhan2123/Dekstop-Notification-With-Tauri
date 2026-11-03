import { useT } from "../../../shared/i18n";
import { useSettingsVM } from "../../settings/viewmodel/useSettingsVM";
import { formatRemaining, usePomodoroVM } from "../viewmodel/usePomodoroVM";

export function PomodoroSettings() {
  const t = useT();
  const vm = usePomodoroVM();
  const settings = useSettingsVM();
  const { phase, remaining_secs } = vm.status;
  const running = phase !== "idle";

  return (
    <fieldset className="field">
      <legend>{t("settings.pomodoro")}</legend>
      <p className="muted">{t("settings.pomodoro.hint")}</p>
      {settings.settings && (
        <label className="row">
          <input
            type="checkbox"
            checked={settings.settings.sound_enabled}
            onChange={(e) => void settings.update({ sound_enabled: e.currentTarget.checked })}
          />
          {t("settings.pomodoro.sound")}
        </label>
      )}
      {settings.settings && (
        <div className="row">
          <label>
            {t("settings.pomodoro.focus_minutes")}
            <input
              type="number"
              min={1}
              max={120}
              value={settings.settings.focus_minutes}
              onChange={(e) =>
                void settings.update({ focus_minutes: Number(e.currentTarget.value) || 1 })
              }
            />
          </label>
          <label>
            {t("settings.pomodoro.break_minutes")}
            <input
              type="number"
              min={1}
              max={60}
              value={settings.settings.break_minutes}
              onChange={(e) =>
                void settings.update({ break_minutes: Number(e.currentTarget.value) || 1 })
              }
            />
          </label>
        </div>
      )}
      <p>
        {running
          ? t(`settings.pomodoro.${phase}`, formatRemaining(remaining_secs))
          : t("settings.pomodoro.idle")}
      </p>
      <div className="row">
        {running ? (
          <button type="button" onClick={vm.stop}>
            {t("settings.pomodoro.stop")}
          </button>
        ) : (
          <button type="button" onClick={() => vm.start()}>
            {t("settings.pomodoro.start")}
          </button>
        )}
      </div>
    </fieldset>
  );
}
