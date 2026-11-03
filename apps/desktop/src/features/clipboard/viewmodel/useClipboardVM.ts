import { useCallback, useEffect, useState } from "react";
import { commands, type ClipEntryDto } from "../../../shared/ipc";

const POLL_MS = 1500;

/** Daftar riwayat clipboard (disegarkan berkala selama jendela Setelan terbuka dan fitur aktif). */
export function useClipboardVM(enabled: boolean) {
  const [query, setQuery] = useState("");
  const [items, setItems] = useState<ClipEntryDto[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => void commands.clipboardList(query).then(setItems), [query]);

  useEffect(() => {
    if (!enabled) {
      setItems([]);
      return;
    }
    refresh();
    const timer = setInterval(refresh, POLL_MS);
    return () => clearInterval(timer);
  }, [enabled, refresh]);

  return {
    query,
    setQuery,
    items,
    error,
    copy: async (id: number) => {
      const r = await commands.clipboardCopy(id);
      setError(r.status === "error" ? r.error : null);
      refresh();
    },
    clear: () => void commands.clipboardClear().then(refresh),
  };
}
