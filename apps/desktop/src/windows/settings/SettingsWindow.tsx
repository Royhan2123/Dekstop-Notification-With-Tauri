import { Onboarding } from "../../features/onboarding/view/Onboarding";
import { SettingsForm } from "../../features/settings/view/SettingsForm";
import { useSettingsStore } from "../../features/settings/store";
import { useSettingsSync } from "../../features/settings/viewmodel/useSettingsSync";
import "../../shared/theme/tokens.css";
import "./settings.css";

export function SettingsWindow() {
  useSettingsSync();
  const onboarded = useSettingsStore((s) => s.settings?.onboarded);
  if (onboarded === undefined) return null;
  return onboarded ? <SettingsForm /> : <Onboarding />;
}
