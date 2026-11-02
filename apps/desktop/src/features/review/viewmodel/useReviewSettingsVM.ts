import { useState } from "react";
import { commands } from "../../../shared/ipc";

/** Daftar repo yang diawasi ada di Settings; menambah/melepas lewat backend agar hook ikut dipasang. */
export function useReviewSettingsVM() {
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function run(call: Promise<{ status: "ok" } | { status: "error"; error: string }>) {
    setBusy(true);
    const r = await call;
    setError(r.status === "ok" ? null : r.error);
    setBusy(false);
    return r.status === "ok";
  }

  return {
    error,
    busy,
    watch: (path: string) => run(commands.reviewWatch(path.trim())),
    unwatch: (path: string) => run(commands.reviewUnwatch(path)),
  };
}
