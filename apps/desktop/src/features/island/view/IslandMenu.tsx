import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { formatRemaining, usePomodoroVM } from "../../pomodoro/viewmodel/usePomodoroVM";
import { useIslandStore } from "../store";
import type { Shape } from "../viewmodel/useIslandVM";

export const menuShape: Shape = { width: 230, height: 188, radius: 20, opacity: 1 };

/** Menu klik kanan: aksi cepat tanpa membuka Setelan. Menutup sendiri saat kursor keluar dari pulau. */
export function IslandMenu({
  onClose,
  onOpenSettings,
}: {
  onClose: () => void;
  onOpenSettings: () => void;
}) {
  const t = useT();
  const dnd = useIslandStore((s) => s.snapshot.dnd);
  const pomodoro = usePomodoroVM();
  const focusing = pomodoro.status.phase !== "idle";

  const run = (action: () => void) => (e: React.MouseEvent) => {
    e.stopPropagation();
    action();
    onClose();
  };

  return (
    <div className="island-menu" role="menu" onContextMenu={(e) => e.preventDefault()}>
      <button type="button" role="menuitemcheckbox" aria-checked={dnd} onClick={run(() => void commands.islandSetDnd(!dnd))}>
        <span>{t("menu.dnd")}</span>
        <span>{dnd ? "✓" : ""}</span>
      </button>
      <button type="button" role="menuitem" onClick={run(focusing ? pomodoro.stop : pomodoro.start)}>
        <span>{focusing ? t("menu.focus_stop") : t("menu.focus_start")}</span>
        <span>{focusing ? formatRemaining(pomodoro.status.remaining_secs) : ""}</span>
      </button>
      <button type="button" role="menuitem" onClick={run(() => void commands.askOpen())}>
        <span>{t("menu.ask")}</span>
      </button>
      <button type="button" role="menuitem" onClick={run(() => void commands.shelfShow())}>
        <span>{t("menu.shelf")}</span>
      </button>
      <button type="button" role="menuitem" onClick={run(onOpenSettings)}>
        <span>{t("menu.settings")}</span>
      </button>
    </div>
  );
}
