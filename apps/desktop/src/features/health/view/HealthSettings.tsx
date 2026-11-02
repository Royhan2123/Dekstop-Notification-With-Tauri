import "./health.css";
import { useT } from "../../../shared/i18n";
import type { HealthRules } from "../../../bindings";
import { useSettingsVM } from "../../settings/viewmodel/useSettingsVM";
import { formatHours, useHealthVM } from "../viewmodel/useHealthVM";

const RULES: (keyof HealthRules)[] = [
  "continuous",
  "twenty_twenty_twenty",
  "bright_night",
  "late_night",
];

/** Aturan pengingat kesehatan, ringkasan hari ini, dan grafik 7 hari. */
export function HealthSettings() {
  const t = useT();
  const vm = useSettingsVM();
  const health = useHealthVM();
  if (!vm.settings) return null;
  const { settings } = vm;

  return (
    <fieldset className="field">
      <legend>{t("settings.health")}</legend>
      <label className="row">
        <input
          type="checkbox"
          checked={settings.health_enabled}
          onChange={(e) => void vm.update({ health_enabled: e.currentTarget.checked })}
        />
        {t("settings.health.enabled")}
      </label>
      {RULES.map((rule) => (
        <label key={rule} className="row">
          <input
            type="checkbox"
            disabled={!settings.health_enabled}
            checked={settings.health_rules[rule]}
            onChange={(e) =>
              void vm.update({
                health_rules: { ...settings.health_rules, [rule]: e.currentTarget.checked },
              })
            }
          />
          {t(`settings.health.rule.${rule}`)}
        </label>
      ))}

      {health.error && <p className="muted">{health.error}</p>}
      {health.summary && (
        <div className="column">
          <span>
            {t("settings.health.today")}:{" "}
            <strong>{formatHours(health.summary.today_active_min)}</strong>
            {health.summary.brightness !== null &&
              ` · ${t("settings.health.brightness")} ${health.summary.brightness}%`}
          </span>
          <div className="week-chart" role="img" aria-label={t("settings.health.week")}>
            {health.bars.map((b) => (
              <div key={b.date} className="week-bar" title={`${b.date}: ${formatHours(b.minutes)}`}>
                <i style={{ height: `${Math.max(b.percent, b.minutes > 0 ? 4 : 0)}%` }} />
                <span>{b.date.slice(8)}</span>
              </div>
            ))}
          </div>
          {health.weekBrightness !== null && (
            <span className="muted">
              {t("settings.health.week_brightness", health.weekBrightness)}
            </span>
          )}
        </div>
      )}
    </fieldset>
  );
}
