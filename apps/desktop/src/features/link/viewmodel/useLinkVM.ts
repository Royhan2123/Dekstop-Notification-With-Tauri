import { useCallback, useEffect, useState } from "react";
import { commands, subscribeIsland } from "../../../shared/ipc";
import { useIslandStore } from "../../island/store";

/** Status pairing/koneksi ekstensi dan tombol Pasangkan / Putuskan di Setelan. */
export function useLinkVM() {
  const connected = useIslandStore((s) => s.snapshot.extension_connected);
  const setSnapshot = useIslandStore((s) => s.setSnapshot);
  const [paired, setPaired] = useState(false);
  const [code, setCode] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const r = await commands.bridgePaired();
    if (r.status === "ok") setPaired(r.data);
    else setError(r.error);
  }, []);

  useEffect(() => {
    const stop = subscribeIsland(setSnapshot);
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, [setSnapshot]);

  // Sambungan berhasil berarti pairing selesai; kode tidak perlu ditampilkan lagi.
  useEffect(() => {
    void refresh();
    if (connected) setCode(null);
  }, [connected, refresh]);

  return {
    paired,
    connected,
    code,
    error,
    pair: async () => setCode(await commands.bridgePair()),
    unpair: async () => {
      const r = await commands.bridgeUnpair();
      if (r.status === "error") setError(r.error);
      setCode(null);
      await refresh();
    },
  };
}
