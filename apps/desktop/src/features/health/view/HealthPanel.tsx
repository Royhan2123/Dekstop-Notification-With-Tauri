import { useT } from "../../../shared/i18n";
import { formatHours, useHealthVM } from "../viewmodel/useHealthVM";

/** Ringkasan kesehatan layar di pulau: jam aktif hari ini, kecerahan, dan batang 7 hari. */
export function HealthPanel({ onClose }: { onClose: () => void }) {
  const t = useT();
  const health = useHealthVM();

  return (
    <div className="health-panel" onClick={(e) => e.stopPropagation()}>
      <div className="shelf-head">
        <span>{t("settings.health")}</span>
        <button type="button" aria-label={t("island.dismiss")} onClick={onClose}>
          ×
        </button>
      </div>
      {health.error && <p className="shelf-empty">{health.error}</p>}
      {health.summary && (
        <>
          <p className="health-today">
            {t("settings.health.today")}:{" "}
            <strong>{formatHours(health.summary.today_active_min)}</strong>
            {health.summary.brightness !== null &&
              ` · ${t("settings.health.brightness")} ${health.summary.brightness}%`}
          </p>
          <div className="health-bars" role="img" aria-label={t("settings.health.week")}>
            {health.bars.map((b) => (
              <div key={b.date} title={`${b.date}: ${formatHours(b.minutes)}`}>
                <i style={{ height: `${Math.max(b.percent, b.minutes > 0 ? 6 : 0)}%` }} />
                <span>{b.date.slice(8)}</span>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
