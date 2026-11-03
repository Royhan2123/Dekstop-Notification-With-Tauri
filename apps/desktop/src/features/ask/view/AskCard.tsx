import { useEffect, useRef } from "react";
import { useT } from "../../../shared/i18n";
import { useAskVM } from "../viewmodel/useAskVM";

export function AskCard() {
  const t = useT();
  const vm = useAskVM();
  const input = useRef<HTMLInputElement>(null);

  // Fokus ke kotak tiap kali kembali ke fase input (dibuka ulang lewat pintasan).
  useEffect(() => {
    if (vm.phase === "input") input.current?.focus();
  }, [vm.phase]);

  return (
    <div className="ask-card" onClick={(e) => e.stopPropagation()}>
      <form
        className="ask-form"
        onSubmit={(e) => {
          e.preventDefault();
          vm.submit();
        }}
      >
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
        <button type="submit" disabled={vm.phase === "thinking" || !vm.question.trim()}>
          {t("ask.send")}
        </button>
        <button type="button" aria-label={t("ask.close")} onClick={vm.close}>
          ×
        </button>
      </form>
      {vm.phase === "thinking" && <p className="ask-status">{t("ask.thinking")}</p>}
      {vm.phase === "done" && <p className="ask-answer">{vm.answer}</p>}
      {vm.phase === "failed" && (
        <p role="alert" className="ask-answer ask-error">
          {vm.answer}
        </p>
      )}
    </div>
  );
}
