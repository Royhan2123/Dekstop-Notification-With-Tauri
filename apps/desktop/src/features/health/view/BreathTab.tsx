import { useT } from "../../../shared/i18n";
import { BREATH_CYCLES, scaleAfter, useBreathVM, type PatternId } from "../viewmodel/useBreathVM";

const PATTERN_KEY = { box: "health.breath.box", calm: "health.breath.calm" } as const;

/** Latihan napas berpemandu: bulatan mengembang saat tarik napas, mengecil saat hembus. */
export function BreathTab({ onDone, count }: { onDone: () => void; count: number }) {
  const t = useT();
  const vm = useBreathVM(onDone);
  const { state, current } = vm;
  const scale = state.done ? 1 : state.running ? scaleAfter(vm.steps, state.step) : 0.7;
  const label = state.done
    ? t("health.breath.done")
    : state.running
      ? t(`health.breath.${current.kind}`)
      : t("health.breath.hint");

  return (
    <div className="breath">
      <div className="breath-seg" role="group">
        {(Object.keys(PATTERN_KEY) as PatternId[]).map((id) => (
          <button
            key={id}
            type="button"
            className={vm.pattern === id ? "on" : ""}
            aria-pressed={vm.pattern === id}
            disabled={state.running}
            onClick={() => vm.setPattern(id)}
          >
            {t(PATTERN_KEY[id])}
          </button>
        ))}
      </div>
      <div className="breath-stage">
        <div className="breath-orb-wrap" aria-hidden="true">
          <div
            className={state.done ? "breath-orb done" : "breath-orb"}
            style={{
              transform: `scale(${scale})`,
              transitionDuration: state.running
                ? `${current.kind === "hold" ? 0.3 : current.secs}s`
                : "0.6s",
            }}
          >
            <span />
            <span />
          </div>
        </div>
        <div className="breath-text" role="status">
          <b key={label}>{label}</b>
          <div className="breath-dots" aria-hidden="true">
            {Array.from({ length: BREATH_CYCLES }, (_, i) => (
              <i key={i} className={i < state.cycle ? "on" : ""} />
            ))}
          </div>
          <small>{t("health.breath.count", count)}</small>
        </div>
        <button type="button" className="primary" onClick={state.running ? vm.stop : vm.start}>
          {state.running ? t("health.breath.stop") : t("health.breath.start")}
        </button>
      </div>
    </div>
  );
}
