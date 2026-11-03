import { useState } from "react";
import { useT } from "../../../shared/i18n";
import { useSettingsStore } from "../../settings/store";
import { formatRemaining, toSeconds, usePomodoroVM } from "../viewmodel/usePomodoroVM";

const clampField = (v: string, max: number) => String(Math.min(max, Math.max(0, Number(v) || 0)));

/** Fokus di pulau: atur jam, menit, detik lalu mulai; selama berjalan tampil hitung mundur. */
export function FocusPanel({ onClose }: { onClose: () => void }) {
  const t = useT();
  const pomodoro = usePomodoroVM();
  const minutes = useSettingsStore((s) => s.settings?.focus_minutes ?? 25);
  const [h, setH] = useState(String(Math.floor(minutes / 60)));
  const [m, setM] = useState(String(minutes % 60));
  const [s, setS] = useState("0");
  const { phase, remaining_secs } = pomodoro.status;
  const running = phase !== "idle";
  const total = toSeconds(Number(h), Number(m), Number(s));

  const field = (label: string, value: string, set: (v: string) => void, max: number) => (
    <label className="focus-field">
      <input
        type="number"
        min={0}
        max={max}
        value={value}
        aria-label={label}
        onChange={(e) => set(clampField(e.currentTarget.value, max))}
      />
      <span>{label}</span>
    </label>
  );

  return (
    <div className="focus-panel" onClick={(e) => e.stopPropagation()}>
      <div className="shelf-head">
        <span>{t("dock.focus")}</span>
        <button type="button" aria-label={t("island.dismiss")} onClick={onClose}>
          ×
        </button>
      </div>
      {running ? (
        <>
          <div className="focus-count" role="timer">
            {formatRemaining(remaining_secs)}
          </div>
          <p className="shelf-empty">{t(`focus.${phase}`)}</p>
          <button type="button" className="focus-go" onClick={() => pomodoro.stop()}>
            {t("settings.pomodoro.stop")}
          </button>
        </>
      ) : (
        <>
          <div className="focus-fields">
            {field(t("focus.hours"), h, setH, 24)}
            {field(t("focus.minutes"), m, setM, 59)}
            {field(t("focus.seconds"), s, setS, 59)}
          </div>
          <button
            type="button"
            className="focus-go"
            disabled={total === 0}
            onClick={() => pomodoro.start(total)}
          >
            {t("settings.pomodoro.start")}
          </button>
        </>
      )}
    </div>
  );
}
