import { useEffect, useState } from "react";
import { useReducedMotion } from "motion/react";
import { useT } from "../../../shared/i18n";
import { useSettingsStore } from "../../settings/store";
import { dayScore, streakDays } from "../viewmodel/score";
import { useCountUp } from "../viewmodel/useCountUp";
import { formatHours, useHealthVM } from "../viewmodel/useHealthVM";
import {
  PREFS_KEY,
  loadPrefs,
  savePrefs,
  type ReminderKind,
  type ReminderPrefs,
} from "../viewmodel/useReminders";
import {
  EYE_GOAL,
  EYE_SECS,
  STRETCH_GOAL,
  WATER_GOAL,
  useWellnessVM,
} from "../viewmodel/useWellnessVM";
import { BreathTab } from "./BreathTab";
import { StretchTab } from "./StretchTab";

/** Batas waktu layar sehat per hari; cincin berubah warna mendekati dan melewatinya. */
const DAILY_GOAL_MIN = 8 * 60;
const RING_R = 30;
const RING_LEN = 2 * Math.PI * RING_R;

type Tab = "summary" | "breath" | "stretch";
const TABS = [
  ["summary", "health.tab.summary"],
  ["breath", "health.tab.breath"],
  ["stretch", "health.tab.stretch"],
] as const;

const REMIND_ICONS: Record<ReminderKind, string> = {
  water: "M12 3s6 6.5 6 11a6 6 0 0 1-12 0c0-4.5 6-11 6-11Z",
  eyes: "M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Zm10 3a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z",
  stretch: "M12 6.5a1.7 1.7 0 1 0 0-.01M5 10l7 2 7-2M12 12v5l-4 4M12 17l4 4",
};

/** Kesehatan di pulau: ringkasan (waktu layar, 7 hari, skor, streak, pengingat), latihan napas, peregangan. */
export function HealthPanel({ onClose }: { onClose: () => void }) {
  const t = useT();
  const health = useHealthVM();
  const well = useWellnessVM();
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const calm = !!systemReduce || settingReduce;
  const [tab, setTab] = useState<Tab>("summary");
  const [shown, setShown] = useState(false);
  const [prefs, setPrefs] = useState<ReminderPrefs>(() => {
    try {
      return loadPrefs(localStorage.getItem(PREFS_KEY));
    } catch {
      return loadPrefs(null);
    }
  });

  // Cincin dan batang tumbuh dari nol begitu panel muncul.
  useEffect(() => {
    const id = requestAnimationFrame(() => setShown(true));
    return () => cancelAnimationFrame(id);
  }, []);

  const today = health.summary?.today_active_min ?? 0;
  const level = today >= DAILY_GOAL_MIN ? "over" : today >= DAILY_GOAL_MIN * 0.8 ? "warn" : "ok";
  const ratio = Math.min(1, today / DAILY_GOAL_MIN);
  const resting = well.eyeLeft !== null;
  const score = dayScore({
    todayMin: today,
    goalMin: DAILY_GOAL_MIN,
    water: well.water,
    waterGoal: WATER_GOAL,
    eyes: well.eyes,
    eyeGoal: EYE_GOAL,
    stretch: well.stretch,
    stretchGoal: STRETCH_GOAL,
  });
  const shownScore = useCountUp(score, calm);
  const streak = streakDays(
    health.bars.map((b) => b.minutes),
    DAILY_GOAL_MIN,
  );

  const toggle = (k: ReminderKind) => {
    const next = { ...prefs, [k]: !prefs[k] };
    setPrefs(next);
    savePrefs(next);
  };

  return (
    <div className={`health-panel${calm ? " calm" : ""}`} onClick={(e) => e.stopPropagation()}>
      <div className="shelf-head">
        <div className="seg" role="tablist">
          {TABS.map(([id, key]) => (
            <button
              key={id}
              type="button"
              role="tab"
              aria-selected={tab === id}
              className={tab === id ? "on" : ""}
              onClick={() => setTab(id)}
            >
              {t(key)}
            </button>
          ))}
        </div>
        <span />
        <button type="button" aria-label={t("island.dismiss")} onClick={onClose}>
          ×
        </button>
      </div>

      {tab === "breath" && <BreathTab onDone={well.addBreath} count={well.breath} />}
      {tab === "stretch" && <StretchTab onDone={well.addStretch} count={well.stretch} />}

      {tab === "summary" &&
        (resting ? (
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

            <div className="health-foot">
              <span
                className={streak > 0 ? "chip streak on" : "chip streak"}
                title={t("health.streak_tip")}
              >
                <svg viewBox="0 0 24 24" width="13" height="13" aria-hidden="true">
                  <path d="M12 3c1 3.5 5 5.5 5 10a5 5 0 0 1-10 0c0-2 1-3 2-4 0 2 1 3 2 3 0-3-1-5 1-9Z" />
                </svg>
                {t("health.streak", streak)}
              </span>
              <span className="chip score" title={t("health.score_tip")}>
                {t("health.score", shownScore)}
              </span>
              <span className="grow" />
              <span className="remind-label">{t("health.remind")}</span>
              {(Object.keys(REMIND_ICONS) as ReminderKind[]).map((k) => (
                <button
                  key={k}
                  type="button"
                  className={prefs[k] ? "remind on" : "remind"}
                  aria-pressed={prefs[k]}
                  title={t(`health.remind_toggle.${k}`)}
                  aria-label={t(`health.remind_toggle.${k}`)}
                  onClick={() => toggle(k)}
                >
                  <svg
                    viewBox="0 0 24 24"
                    width="15"
                    height="15"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.8"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <path d={REMIND_ICONS[k]} />
                  </svg>
                </button>
              ))}
            </div>
          </>
        ))}
    </div>
  );
}
