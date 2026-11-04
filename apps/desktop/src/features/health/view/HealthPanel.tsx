import { useEffect, useState } from "react";
import { useReducedMotion } from "motion/react";
import { useT, type MessageKey } from "../../../shared/i18n";
import { useSettingsStore } from "../../settings/store";
import { formatHours, useHealthVM } from "../viewmodel/useHealthVM";
import { EYE_SECS, WATER_GOAL, useWellnessVM } from "../viewmodel/useWellnessVM";

/** Batas waktu layar sehat per hari; cincin berubah warna mendekati dan melewatinya. */
const DAILY_GOAL_MIN = 8 * 60;
const RING_R = 30;
const RING_LEN = 2 * Math.PI * RING_R;
/** Tips berganti sendiri tiap beberapa detik selama panel terbuka. */
const TIP_MS = 9000;
const TIPS = [
  "health.tip.1",
  "health.tip.2",
  "health.tip.3",
  "health.tip.4",
] as const satisfies readonly MessageKey[];

/** Ringkasan kesehatan di pulau: waktu layar hari ini, 7 hari, istirahat mata, air minum, dan tips. */
export function HealthPanel({ onClose }: { onClose: () => void }) {
  const t = useT();
  const health = useHealthVM();
  const well = useWellnessVM();
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const [shown, setShown] = useState(false);
  const [tip, setTip] = useState(() => Math.floor(Math.random() * TIPS.length));

  // Cincin dan batang tumbuh dari nol begitu panel muncul.
  useEffect(() => {
    const id = requestAnimationFrame(() => setShown(true));
    return () => cancelAnimationFrame(id);
  }, []);
  useEffect(() => {
    const id = window.setInterval(() => setTip((n) => (n + 1) % TIPS.length), TIP_MS);
    return () => window.clearInterval(id);
  }, []);

  const today = health.summary?.today_active_min ?? 0;
  const level = today >= DAILY_GOAL_MIN ? "over" : today >= DAILY_GOAL_MIN * 0.8 ? "warn" : "ok";
  const ratio = Math.min(1, today / DAILY_GOAL_MIN);
  const resting = well.eyeLeft !== null;

  return (
    <div
      className={`health-panel${systemReduce || settingReduce ? " calm" : ""}`}
      onClick={(e) => e.stopPropagation()}
    >
      <div className="shelf-head">
        <span>{t("settings.health")}</span>
        <button type="button" aria-label={t("island.dismiss")} onClick={onClose}>
          ×
        </button>
      </div>

      {resting ? (
        <div className="eye-rest" role="status">
          <div className={well.eyeDone ? "orb done" : "orb"} aria-hidden="true">
            <span />
            <span />
          </div>
          <div className="eye-text">
            <b>
              {well.eyeDone ? t("health.eye_done") : t("health.eye_running", well.eyeLeft ?? 0)}
            </b>
            <small>{t("health.eye_hint", EYE_SECS)}</small>
          </div>
          <button type="button" onClick={well.cancelEye}>
            {t("health.eye_cancel")}
          </button>
        </div>
      ) : (
        <>
          {health.error && <p className="shelf-empty">{health.error}</p>}
          <div className="health-hero">
            <svg className="health-ring" viewBox="0 0 72 72" aria-hidden="true">
              <circle className="track" cx="36" cy="36" r={RING_R} />
              <circle
                className={`fill ${level}`}
                cx="36"
                cy="36"
                r={RING_R}
                strokeDasharray={RING_LEN}
                strokeDashoffset={shown ? RING_LEN * (1 - ratio) : RING_LEN}
                transform="rotate(-90 36 36)"
              />
            </svg>
            <div className="health-hero-text">
              <strong>{formatHours(today)}</strong>
              <small>{t("health.goal", formatHours(DAILY_GOAL_MIN))}</small>
              {health.summary?.brightness != null && (
                <small>
                  {t("settings.health.brightness")} {health.summary.brightness}%
                </small>
              )}
            </div>
            <div className="health-bars" role="img" aria-label={t("settings.health.week")}>
              {health.bars.map((b, i) => (
                <div key={b.date} title={`${b.date}: ${formatHours(b.minutes)}`}>
                  <i
                    style={
                      {
                        height: `${Math.max(b.percent, b.minutes > 0 ? 8 : 0)}%`,
                        "--i": i,
                      } as React.CSSProperties
                    }
                  />
                  <span>{b.date.slice(8)}</span>
                </div>
              ))}
            </div>
          </div>

          <div className="health-actions">
            <button type="button" className="health-eye" onClick={well.startEye}>
              {t("health.eye_start")}
              {well.eyes > 0 && <em>{well.eyes}×</em>}
            </button>
            <button type="button" className="health-water" onClick={well.addWater}>
              <span className="drops" aria-hidden="true">
                {Array.from({ length: WATER_GOAL }, (_, i) => (
                  <i key={i} className={i < well.water ? "on" : ""} />
                ))}
              </span>
              {t("health.water_add")} · {Math.min(well.water, WATER_GOAL)}/{WATER_GOAL}
            </button>
          </div>

          <p className="health-tip" key={tip}>
            {t(TIPS[tip])}
          </p>
        </>
      )}
    </div>
  );
}
