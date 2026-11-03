import { useState } from "react";
import { commands } from "../../../shared/ipc";
import { useIslandStore } from "../../island/store";

export type AskPhase = "input" | "thinking" | "done" | "failed";

/** Quick Ask: fase dan jawaban datang dari snapshot pulau; pertanyaan hanya ada di sini. */
export function useAskVM() {
  const content = useIslandStore((s) => s.snapshot.content);
  const [question, setQuestion] = useState("");
  const phase = (content?.detail ?? "input") as AskPhase;

  return {
    phase,
    answer: content?.text ?? "",
    question,
    setQuestion,
    submit: () => {
      if (question.trim()) void commands.askSubmit(question);
    },
    close: () => void commands.islandDismiss(),
  };
}
