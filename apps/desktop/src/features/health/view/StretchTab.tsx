import { useT } from "../../../shared/i18n";
import {
  STRETCH_MOVES,
  STRETCH_SECS,
  useStretchVM,
  type StretchMove,
} from "../viewmodel/useStretchVM";

const NAME = {
  neck: "health.stretch.neck",
  shoulders: "health.stretch.shoulders",
  wrists: "health.stretch.wrists",
  back: "health.stretch.back",
} as const;

const HINT = {
  neck: "health.stretch.neck_hint",
  shoulders: "health.stretch.shoulders_hint",
  wrists: "health.stretch.wrists_hint",
  back: "health.stretch.back_hint",
} as const;

/** Figur garis sederhana; tiap gerakan menganimasikan bagian tubuhnya lewat CSS. */
function Figure({ move }: { move: StretchMove }) {
  return (
    <svg className={`fig ${move}`} viewBox="0 0 64 64" aria-hidden="true">
      <g className="fig-upper">
        <g className="fig-head">
          <circle cx="32" cy="13" r="6" />
          <path d="M32 19v4" />
        </g>
        <path d="M32 23v17" />
        <g className="fig-sh">
          <path d="M21 26h22" />
          <path d="M21 26l-5 9" />
          <path className="fig-fore l" d="M16 35l-3 8" />
          <path d="M43 26l5 9" />
          <path className="fig-fore r" d="M48 35l3 8" />
        </g>
      </g>
      <path d="M32 40l-7 16M32 40l7 16" />
    </svg>
  );
}

/** Peregangan berpemandu: empat gerakan masing-masing 20 detik, bisa dilewati. */
export function StretchTab({ onDone, count }: { onDone: () => void; count: number }) {
  const t = useT();
  const vm = useStretchVM(onDone);
  const { state, move } = vm;
  const elapsed = (STRETCH_SECS - state.left + 1) / STRETCH_SECS;

  return (
    <div className="stretch" role="status">
      <div className={state.done ? "stretch-fig done" : "stretch-fig"}>
        <Figure move={state.running ? move : "neck"} />
      </div>
      <div className="stretch-text">
        {state.done ? (
          <b>{t("health.stretch.done")}</b>
        ) : state.running ? (
          <>
            <b key={move}>{t(NAME[move])}</b>
            <small key={`${move}-hint`}>{t(HINT[move])}</small>
            <div className="stretch-bar" aria-hidden="true">
              <i style={{ width: `${Math.min(100, elapsed * 100)}%` }} />
            </div>
            <div className="stretch-dots" aria-hidden="true">
              {STRETCH_MOVES.map((m, i) => (
                <i key={m} className={i < state.move ? "on" : i === state.move ? "now" : ""} />
              ))}
              <span>{t("health.stretch.step", state.move + 1, STRETCH_MOVES.length)}</span>
            </div>
          </>
        ) : (
          <>
            <b>{t("health.stretch.title")}</b>
            <small>{t("health.stretch.intro", STRETCH_MOVES.length, STRETCH_SECS)}</small>
            <small className="count">{t("health.stretch.count", count)}</small>
          </>
        )}
      </div>
      <div className="stretch-actions">
        {state.running ? (
          <>
            <button type="button" onClick={vm.skip}>
              {t("health.stretch.skip")}
            </button>
            <button type="button" onClick={vm.stop}>
              {t("health.breath.stop")}
            </button>
          </>
        ) : (
          <button type="button" className="primary" onClick={vm.start}>
            {t("health.stretch.start")}
          </button>
        )}
      </div>
    </div>
  );
}
