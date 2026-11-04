import { useState } from "react";
import { commands } from "../../../shared/ipc";
import { useIslandStore } from "../../island/store";
import { useChatStore } from "../store";

export type AskPhase = "input" | "thinking" | "done" | "failed";

/** Quick Ask: fase dan galat datang dari snapshot pulau, riwayat dari backend; pertanyaan yang diketik hanya ada di sini. */
export function useAskVM() {
  const content = useIslandStore((s) => s.snapshot.content);
  const turns = useChatStore((s) => s.turns);
  const setTurns = useChatStore((s) => s.setTurns);
  const [question, setQuestion] = useState("");
  // Pertanyaan yang sedang dijawab; riwayat baru memuatnya setelah jawabannya datang.
  const [pending, setPending] = useState("");
  const phase = (content?.detail ?? "input") as AskPhase;

  return {
    phase,
    /** Teks galat bila fase "failed"; jawaban yang berhasil ada di riwayat. */
    error: phase === "failed" ? (content?.text ?? "") : "",
    turns,
    pending,
    question,
    setQuestion,
    submit: () => {
      const q = question.trim();
      if (!q) return;
      setPending(q);
      setQuestion("");
      void commands.askSubmit(q);
    },
    clear: async () => {
      await commands.askClear();
      setTurns([]);
      // Fase kembali ke kotak tulis supaya jawaban terakhir yang sudah dihapus tidak tertinggal.
      await commands.askOpen();
    },
    close: () => void commands.islandDismiss(),
  };
}
