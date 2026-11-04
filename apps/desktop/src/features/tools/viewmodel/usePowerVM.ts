import { useEffect, useReducer, useState } from "react";
import { commands, type PowerActionDto } from "../../../shared/ipc";

/** Detik hitung mundur setelah pengguna menyetujui; masih bisa dibatalkan sampai habis. */
export const COUNTDOWN_SECS = 5;

export type PowerStep =
  | { phase: "idle" }
  | { phase: "confirm"; action: PowerActionDto }
  | { phase: "counting"; action: PowerActionDto; left: number };

export type PowerEvent =
  | { type: "pick"; action: PowerActionDto }
  | { type: "confirm" }
  | { type: "tick" }
  | { type: "cancel" };

/** Mesin keadaan murni: pilih → konfirmasi → hitung mundur → (jalankan saat left = 0). Kunci layar tidak lewat sini. */
export function powerReducer(s: PowerStep, e: PowerEvent): PowerStep {
  switch (e.type) {
    case "pick":
      return e.action === "lock" ? s : { phase: "confirm", action: e.action };
    case "confirm":
      return s.phase === "confirm"
        ? { phase: "counting", action: s.action, left: COUNTDOWN_SECS }
        : s;
    case "tick":
      return s.phase === "counting" && s.left > 0 ? { ...s, left: s.left - 1 } : s;
    case "cancel":
      return { phase: "idle" };
  }
}

/** Perintah daya dengan kartu "Yakin?" dan hitung mundur yang bisa dibatalkan. */
export function usePowerVM() {
  const [step, dispatch] = useReducer(powerReducer, { phase: "idle" });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (step.phase !== "counting") return;
    if (step.left > 0) {
      const id = window.setTimeout(() => dispatch({ type: "tick" }), 1000);
      return () => window.clearTimeout(id);
    }
    void commands.powerPerform(step.action).then((r) => {
      setError(r.status === "error" ? r.error : null);
      // Setelah tidur, pengguna kembali ke panel yang bersih.
      dispatch({ type: "cancel" });
    });
  }, [step]);

  return {
    step,
    error,
    pick: (action: PowerActionDto) => {
      setError(null);
      if (action === "lock") {
        void commands.powerPerform("lock").then((r) => {
          if (r.status === "error") setError(r.error);
        });
      } else dispatch({ type: "pick", action });
    },
    confirm: () => dispatch({ type: "confirm" }),
    cancel: () => dispatch({ type: "cancel" }),
  };
}
