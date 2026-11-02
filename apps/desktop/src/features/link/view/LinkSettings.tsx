import { useT } from "../../../shared/i18n";
import { useLinkVM } from "../viewmodel/useLinkVM";

/** Pairing ekstensi browser Isla Link. */
export function LinkSettings() {
  const t = useT();
  const vm = useLinkVM();
  const status = vm.connected
    ? t("settings.link.connected")
    : vm.paired
      ? `${t("settings.link.paired")} · ${t("settings.link.offline")}`
      : t("settings.link.unpaired");

  return (
    <fieldset className="field">
      <legend>{t("settings.link")}</legend>
      <p className="muted">{t("settings.link.hint")}</p>
      <p>{status}</p>
      {vm.code && <p className="pair-code">{t("settings.link.code", vm.code)}</p>}
      <div className="row">
        <button type="button" onClick={() => void vm.pair()}>
          {t("settings.link.pair")}
        </button>
        {vm.paired && (
          <button type="button" onClick={() => void vm.unpair()}>
            {t("settings.link.unpair")}
          </button>
        )}
      </div>
      {vm.error && (
        <p role="alert" className="error">
          {t("settings.error", vm.error)}
        </p>
      )}
    </fieldset>
  );
}
