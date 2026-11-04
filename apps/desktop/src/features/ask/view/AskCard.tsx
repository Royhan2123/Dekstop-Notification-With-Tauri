import { useEffect, useRef } from "react";
import { useReducedMotion } from "motion/react";
import { useT } from "../../../shared/i18n";
import { useSettingsStore } from "../../settings/store";
import { useAskVM } from "../viewmodel/useAskVM";

/** Jawaban muncul kata demi kata; kata setelah ini muncul bersamaan supaya jawaban panjang tidak lambat. */
const MAX_STAGGER_WORDS = 60;

function Reveal({ text }: { text: string }) {
  let word = 0;
  return (
    <>
      {text.split(/(\s+)/).map((part, i) =>
        /^\s*$/.test(part) ? (
          part
        ) : (
          <span
            key={i}
            className="ask-word"
            style={{ "--i": Math.min(word++, MAX_STAGGER_WORDS) } as React.CSSProperties}
          >
            {part}
          </span>
        ),
      )}
    </>
  );
}

export function AskCard() {
  const t = useT();
  const vm = useAskVM();
  const input = useRef<HTMLInputElement>(null);
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);

  // Fokus ke kotak tiap kali kembali ke fase input (dibuka ulang lewat pintasan).
  useEffect(() => {
    if (vm.phase === "input") input.current?.focus();
  }, [vm.phase]);

  return (
    <div
      className={`ask-card${systemReduce || settingReduce ? " calm" : ""}`}
      data-phase={vm.phase}
      onClick={(e) => e.stopPropagation()}
    >
      <form
        className="ask-form"
        onSubmit={(e) => {
          e.preventDefault();
          vm.submit();
        }}
      >
        <span className="ask-field">
          <input
            ref={input}
            autoFocus
            value={vm.question}
            placeholder={t("ask.placeholder")}
            disabled={vm.phase === "thinking"}
            onChange={(e) => vm.setQuestion(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") vm.close();
            }}
          />
        </span>
        <button type="submit" disabled={vm.phase === "thinking" || !vm.question.trim()}>
          {t("ask.send")}
        </button>
        <button type="button" aria-label={t("ask.close")} onClick={vm.close}>
          ×
        </button>
      </form>
      {vm.phase === "thinking" && (
        <div className="ask-status" role="status">
          <span className="ask-dots" aria-hidden="true">
            <i />
            <i />
            <i />
          </span>
          {t("ask.thinking")}
          <span className="ask-shimmer" aria-hidden="true" />
        </div>
      )}
      {vm.phase === "done" && (
        <p className="ask-answer" key={vm.answer}>
          <Reveal text={vm.answer} />
        </p>
      )}
      {vm.phase === "failed" && (
        <p role="alert" className="ask-answer ask-error">
          {vm.answer}
        </p>
      )}
    </div>
  );
}
