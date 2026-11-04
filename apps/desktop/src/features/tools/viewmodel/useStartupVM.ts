import { useCallback, useEffect, useState } from "react";
import { commands, type InstalledAppDto, type StartupAppDto } from "../../../shared/ipc";
import { useSettingsStore } from "../../settings/store";

type Done = { status: "ok" } | { status: "error"; error: string };

/** Saring daftar aplikasi terpasang menurut kata kunci (huruf besar/kecil diabaikan). */
export function filterApps(apps: InstalledAppDto[], query: string): InstalledAppDto[] {
  const q = query.trim().toLowerCase();
  return q ? apps.filter((a) => a.name.toLowerCase().includes(q)) : apps;
}

/** Daftar aplikasi yang dibuka saat Windows menyala + saklar "buka Isla Desk saat Windows menyala". */
export function useStartupVM() {
  const [apps, setApps] = useState<StartupAppDto[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [started, setStarted] = useState(false);
  /** `null` = pemilih tertutup; larik = pemilih terbuka (kosong selagi memuat). */
  const [installed, setInstalled] = useState<InstalledAppDto[] | null>(null);
  const [loading, setLoading] = useState(false);
  const settings = useSettingsStore((s) => s.settings);
  const setSettings = useSettingsStore((s) => s.setSettings);

  const refresh = useCallback(async () => {
    const r = await commands.startupList();
    if (r.status === "ok") setApps(r.data);
    else setError(r.error);
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const run = async (op: Promise<Done>) => {
    const r = await op;
    setError(r.status === "error" ? r.error : null);
    await refresh();
    return r.status === "ok";
  };

  return {
    apps,
    error,
    started,
    autostart: settings?.autostart ?? false,
    setAutostart: async (on: boolean) => {
      if (!settings) return;
      const r = await commands.settingsUpdate({ ...settings, autostart: on });
      if (r.status === "ok") setSettings(r.data);
      else setError(r.error);
    },
    installed,
    loading,
    /** Buka pemilih: aplikasi terpasang dibaca dari Windows. */
    openPicker: async () => {
      setInstalled([]);
      setLoading(true);
      const r = await commands.startupInstalled();
      setLoading(false);
      if (r.status === "ok") setInstalled(r.data);
      else {
        setError(r.error);
        setInstalled(null);
      }
    },
    closePicker: () => setInstalled(null),
    add: (a: InstalledAppDto) => run(commands.startupAdd(a.name, a.path)),
    remove: (a: StartupAppDto) => run(commands.startupRemove(a.id)),
    move: (a: StartupAppDto, up: boolean) => run(commands.startupMove(a.id, up)),
    setDelay: (a: StartupAppDto, secs: number) =>
      run(commands.startupSetDelay(a.id, Math.max(0, Math.round(secs) || 0))),
    runNow: async () => {
      const ok = await run(commands.startupRun());
      setStarted(ok);
    },
  };
}
