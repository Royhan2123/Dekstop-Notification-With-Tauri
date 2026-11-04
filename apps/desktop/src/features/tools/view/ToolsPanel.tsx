import type { CSSProperties } from "react";
import { useReducedMotion } from "motion/react";
import { useT, type MessageKey } from "../../../shared/i18n";
import type { PowerActionDto } from "../../../shared/ipc";
import { useSettingsStore } from "../../settings/store";
import { COUNTDOWN_SECS, usePowerVM } from "../viewmodel/usePowerVM";

const POWER_ICONS: Record<PowerActionDto, React.ReactNode> = {
  shutdown: <path d="M12 3v9M6.3 6.8a8 8 0 1 0 11.4 0" />,
  restart: <path d="M20 12a8 8 0 1 1-2.6-5.9M20 4v4.5h-4.5" />,
  sleep: <path d="M20 15.5A8.5 8.5 0 0 1 8.5 4a8.5 8.5 0 1 0 11.5 11.5ZM15 4h4l-4 4h4" />,
  lock: (
    <>
      <rect x="5" y="11" width="14" height="9" rx="2" />
      <path d="M8 11V8a4 4 0 0 1 8 0v3" />
    </>
  ),
};

const ACTIONS: PowerActionDto[] = ["shutdown", "restart", "sleep", "lock"];

const LABEL = {
  shutdown: "tools.power.shutdown",
  restart: "tools.power.restart",
  sleep: "tools.power.sleep",
  lock: "tools.power.lock",
} as const satisfies Record<PowerActionDto, MessageKey>;

const ASK = {
  shutdown: "tools.confirm.shutdown",
  restart: "tools.confirm.restart",
  sleep: "tools.confirm.sleep",
  lock: "tools.confirm.sleep",
} as const satisfies Record<PowerActionDto, MessageKey>;

const DOING = {
  shutdown: "tools.doing.shutdown",
  restart: "tools.doing.restart",
  sleep: "tools.doing.sleep",
  lock: "tools.doing.sleep",
} as const satisfies Record<PowerActionDto, MessageKey>;

const RING_R = 28;
const RING_LEN = 2 * Math.PI * RING_R;

function Glyph({ action, size = 22 }: { action: PowerActionDto; size?: number }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width={size}
      height={size}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {POWER_ICONS[action]}
    </svg>
  );
}

/** Menu Alat di pulau. Sekarang berisi Daya (matikan, mulai ulang, tidur, kunci) dengan konfirmasi. */
export function ToolsPanel({ onClose }: { onClose: () => void }) {
  const t = useT();
  const power = usePowerVM();
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const { step } = power;

  return (
    <div
      className={`tools-panel${systemReduce || settingReduce ? " calm" : ""}`}
      onClick={(e) => e.stopPropagation()}
    >
      <div className="shelf-head">
        <span>{t("tools.title")}</span>
        <button type="button" aria-label={t("island.dismiss")} onClick={onClose}>
          ×
        </button>
      </div>

      {step.phase === "idle" && (
        <>
          <p className="tools-sec">{t("tools.power")}</p>
          <div className="tool-grid">
            {ACTIONS.map((a, i) => (
              <button
                key={a}
                type="button"
                className={`tool-tile power-${a}`}
                style={{ "--i": i } as CSSProperties}
                onClick={() => power.pick(a)}
              >
                <Glyph action={a} />
                <span>{t(LABEL[a])}</span>
              </button>
            ))}
          </div>
          {power.error && <p className="shelf-empty">{power.error}</p>}
        </>
      )}

      {step.phase === "confirm" && (
        <div className="power-confirm" role="alertdialog">
          <div className={`power-icon power-${step.action}`}>
            <Glyph action={step.action} size={26} />
          </div>
          <div className="power-text">
            <b>{t(ASK[step.action])}</b>
            <small>{t("tools.confirm_hint", COUNTDOWN_SECS)}</small>
          </div>
          <div className="power-actions">
            <button type="button" onClick={power.cancel}>
              {t("tools.cancel")}
            </button>
            <button type="button" className="danger" onClick={power.confirm}>
              {t("tools.confirm_yes")}
            </button>
          </div>
        </div>
      )}

      {step.phase === "counting" && (
        <div className="power-confirm" role="status">
          <div className="power-count">
            <svg viewBox="0 0 64 64" aria-hidden="true">
              <circle className="track" cx="32" cy="32" r={RING_R} />
              <circle
                className="fill"
                cx="32"
                cy="32"
                r={RING_R}
                strokeDasharray={RING_LEN}
                transform="rotate(-90 32 32)"
                style={{ animationDuration: `${COUNTDOWN_SECS}s` }}
              />
            </svg>
            <strong>{step.left}</strong>
          </div>
          <div className="power-text">
            <b>{t(DOING[step.action], step.left)}</b>
            <small>{t("tools.counting_hint")}</small>
          </div>
          <div className="power-actions">
            <button type="button" className="danger" onClick={power.cancel}>
              {t("tools.cancel")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
