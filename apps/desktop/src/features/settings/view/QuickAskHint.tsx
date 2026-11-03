import { useT } from "../../../shared/i18n";

export function QuickAskHint() {
  const t = useT();
  return (
    <fieldset className="field">
      <legend>{t("settings.ask")}</legend>
      <p className="muted">{t("settings.ask.hint")}</p>
    </fieldset>
  );
}
