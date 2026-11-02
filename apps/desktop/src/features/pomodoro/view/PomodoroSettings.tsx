import { useT } from "../../../shared/i18n";
import { formatRemaining, usePomodoroVM } from "../viewmodel/usePomodoroVM";

export function PomodoroSettings() {
  const t = useT();
  const vm = usePomodoroVM();
  const { phase, remaining_secs } = vm.status;
  const running = phase !== "idle";

  return (
    <fieldset className="field">
      <legend>{t("settings.pomodoro")}</legend>
      <p className="muted">{t("settings.pomodoro.hint")}</p>
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
          <button type="button" onClick={vm.start}>
            {t("settings.pomodoro.start")}
          </button>
        )}
      </div>
    </fieldset>
  );
}
