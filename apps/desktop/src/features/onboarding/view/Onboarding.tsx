import { useState } from "react";
import { useT } from "../../../shared/i18n";
import { AiBrainPicker } from "../../settings/view/AiBrainPicker";
import { useSettingsVM } from "../../settings/viewmodel/useSettingsVM";

const STEPS = ["welcome", "ai", "permissions", "autostart"] as const;

export function Onboarding() {
  const t = useT();
  const vm = useSettingsVM();
  const [index, setIndex] = useState(0);
  if (!vm.settings) return null;
  const step = STEPS[index];
  const last = index === STEPS.length - 1;

  return (
    <div className="settings onboarding">
      <p className="muted">{t("onboarding.step", index + 1, STEPS.length)}</p>
      <h1>{t(`onboarding.${step}.title`)}</h1>
      <p>{t(`onboarding.${step}.body`)}</p>

      {step === "ai" && <AiBrainPicker />}
      {step === "autostart" && (
        <label className="row">
          <input
            type="checkbox"
            checked={vm.settings.autostart}
            onChange={(e) => void vm.update({ autostart: e.currentTarget.checked })}
          />
          {t("settings.autostart")}
        </label>
      )}

      {vm.error && (
        <p role="alert" className="error">
          {t("settings.error", vm.error)}
        </p>
      )}

      <div className="row actions">
        {!last && (
          <button type="button" onClick={() => void vm.finishOnboarding()}>
            {t("onboarding.skip")}
          </button>
        )}
        {index > 0 && (
          <button type="button" onClick={() => setIndex(index - 1)}>
            {t("onboarding.back")}
          </button>
        )}
        {last ? (
          <button type="button" className="primary" onClick={() => void vm.finishOnboarding()}>
            {t("onboarding.finish")}
          </button>
        ) : (
          <button type="button" className="primary" onClick={() => setIndex(index + 1)}>
            {t("onboarding.next")}
          </button>
        )}
      </div>
    </div>
  );
}
