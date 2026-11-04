import { useEffect, useRef, useState } from "react";
import { useReducedMotion } from "motion/react";
import { useT } from "../../../shared/i18n";
import { useSettingsStore } from "../../settings/store";
import { useAskVM } from "../viewmodel/useAskVM";
import { AiSwitch } from "./AiSwitch";

/** Jawaban muncul kata demi kata; kata setelah ini muncul bersamaan supaya jawaban panjang tidak lambat. */
const MAX_STAGGER_WORDS = 60;
/** Klik kedua tombol hapus harus datang dalam waktu ini. */
const CONFIRM_MS = 3000;

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
  const log = useRef<HTMLDivElement>(null);
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const [confirming, setConfirming] = useState(false);
  // Hanya jawaban yang datang selama kartu ini terbuka yang dianimasikan; riwayat lama tampil biasa.
  const seen = useRef(vm.turns.length);

  // Fokus ke kotak tiap kali siap menerima pertanyaan (dibuka, atau setelah jawaban/galat).
  useEffect(() => {
    if (vm.phase !== "thinking") input.current?.focus();
  }, [vm.phase]);

  // Gulir ke chat terbaru saat dibuka dan tiap ada giliran baru.
  useEffect(() => {
    const el = log.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [vm.turns.length, vm.phase]);

  useEffect(() => {
    if (!confirming) return;
    const id = window.setTimeout(() => setConfirming(false), CONFIRM_MS);
    return () => window.clearTimeout(id);
  }, [confirming]);

  const thinking = vm.phase === "thinking";
  const showLog = vm.turns.length > 0;

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
            disabled={thinking}
            onChange={(e) => vm.setQuestion(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") vm.close();
            }}
          />
        </span>
        <button type="submit" disabled={thinking || !vm.question.trim()}>
          {t("ask.send")}
        </button>
        <button type="button" aria-label={t("ask.close")} onClick={vm.close}>
          ×
        </button>
      </form>
      {showLog && (
        <div className="ask-log" ref={log}>
          {vm.turns.map((turn, i) => (
            <div className="ask-turn" key={i}>
              <p className="ask-q">{turn.question}</p>
              <p className="ask-a">
                {i >= seen.current ? <Reveal text={turn.answer} /> : turn.answer}
              </p>
              <span className="ask-model">{turn.by}</span>
            </div>
          ))}
          {thinking && vm.pending && <p className="ask-q">{vm.pending}</p>}
        </div>
      )}
      {thinking && (
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
      {vm.error && (
        <p role="alert" className="ask-answer ask-error">
          {vm.error}
        </p>
      )}
      {!thinking && (
        <div className="ask-foot">
          <AiSwitch label={t("ask.use")} />
          {showLog && (
            <button
              type="button"
              className={confirming ? "ask-clear confirming" : "ask-clear"}
              onClick={() => {
                if (!confirming) return setConfirming(true);
                setConfirming(false);
                void vm.clear();
              }}
            >
              {confirming ? t("ask.clear_confirm") : t("ask.clear")}
            </button>
          )}
        </div>
      )}
    </div>
  );
}
