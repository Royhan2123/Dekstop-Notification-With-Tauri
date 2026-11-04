import { useEffect } from "react";
import { commands } from "../../../shared/ipc";
import { useIslandStore } from "../../island/store";
import { useChatStore } from "../store";

/**
 * Memuat riwayat chat saat aplikasi mulai dan tiap fase Tanya AI berganti (jawaban baru masuk riwayat
 * sebelum fase "done" dikirim). Dipasang sekali di pulau supaya ukuran kartu tahu ada riwayat atau tidak.
 */
export function useChatSync(): void {
  const setTurns = useChatStore((s) => s.setTurns);
  const phase = useIslandStore((s) =>
    s.snapshot.content?.kind === "ask" ? (s.snapshot.content.detail ?? "input") : "",
  );

  useEffect(() => {
    let alive = true;
    void commands.askHistory().then((r) => {
      if (alive && r.status === "ok") setTurns(r.data);
    });
    return () => {
      alive = false;
    };
  }, [phase, setTurns]);
}
