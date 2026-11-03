import { useT } from "../../../shared/i18n";

export function TerminalNotifyHint() {
  const t = useT();
  return (
    <fieldset className="field">
      <legend>{t("settings.terminal")}</legend>
      <p className="muted">{t("settings.terminal.hint")}</p>
      <code>npm run build; isla-desk notify &quot;build selesai&quot;</code>
    </fieldset>
  );
}
