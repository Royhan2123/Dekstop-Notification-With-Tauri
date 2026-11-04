import type { CSSProperties } from "react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useT, type MessageKey } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { instant } from "../../../shared/motion/springs";
import { formatRemaining, usePomodoroVM } from "../../pomodoro/viewmodel/usePomodoroVM";
import { useSettingsStore } from "../../settings/store";
import { useIslandStore } from "../store";

export type Panel = "clipboard" | "health" | "net" | "tasks" | "focus" | "tools" | null;

const ICONS = {
  music: (
    <path d="M9 18V5l11-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0Zm11-2a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
  ),
  net: <path d="M12 4v13m0 0-4-4m4 4 4-4M5 20h14" />,
  tasks: <path d="m4 7 2 2 3-4M12 7h8M4 16l2 2 3-4M12 16h8" />,
  health: <path d="M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.5A4 4 0 0 1 19 10c0 5.6-7 10-7 10Z" />,
  dnd: <path d="M20 15.5A8.5 8.5 0 0 1 8.5 4a8.5 8.5 0 1 0 11.5 11.5Z" />,
  focus: (
    <>
      <circle cx="12" cy="13" r="8" />
      <path d="M12 9v4l2.5 2M9 2h6" />
    </>
  ),
  ask: <path d="M4 5h16v11H9l-5 4Z" />,
  shelf: <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z" />,
  clipboard: (
    <path d="M9 4h6v3H9ZM7 5H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-1" />
  ),
  tools: (
    <path d="M5 4h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1Zm9 0h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1ZM5 13h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1Zm9 0h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1Z" />
  ),
  settings: (
    <>
      <path d="M4 7h9M19 7h1M4 17h1M11 17h9" />
      <circle cx="16" cy="7" r="2.4" />
      <circle cx="8" cy="17" r="2.4" />
    </>
  ),
};

function Icon({ name }: { name: keyof typeof ICONS }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {ICONS[name]}
    </svg>
  );
}

/** Pegas highlight aktif yang meluncur antar tombol dan label yang melebar. */
const glide = { type: "spring", stiffness: 520, damping: 38, mass: 0.8 } as const;

interface Item {
  id: keyof typeof ICONS;
  label: MessageKey;
  /** Teks di tombol bila bukan label (mis. hitung mundur); tampil terus walau tidak aktif. */
  text?: string;
  /** Tombol yang sedang "dipegang" highlight meluncur (satu saja pada satu waktu). */
  active: boolean;
  /** Sakelar menyala yang tidak memegang highlight. */
  lit?: boolean;
  disabled?: boolean;
  run: () => void;
}

/**
 * Dok fitur di dasar pulau yang terbuka: satu baris ikon dalam wadah kaca. Tombol aktif memegang
 * highlight beraksen yang meluncur dan memperlihatkan labelnya; yang lain menampilkan label lewat
 * tooltip saat disorot. Di dok sempit label hilang dan baris bisa digulir.
 */
export function IslandDock({
  panel,
  onPanel,
  onOpenSettings,
}: {
  panel: Panel;
  onPanel: (p: Panel) => void;
  onOpenSettings: () => void;
}) {
  const t = useT();
  const snapshot = useIslandStore((s) => s.snapshot);
  const pomodoro = usePomodoroVM();
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const calm = systemReduce || settingReduce;
  const kind = snapshot.content?.kind;
  const focusing = pomodoro.status.phase !== "idle";

  const items: Item[] = [
    {
      id: "music",
      label: "dock.music",
      active: kind === "media" && !panel,
      disabled: !snapshot.has_media,
      run: () => {
        onPanel(null);
        void commands.islandShowMedia();
      },
    },
    {
      id: "health",
      label: "dock.health",
      active: panel === "health",
      run: () => onPanel(panel === "health" ? null : "health"),
    },
    {
      id: "dnd",
      label: "dock.dnd",
      active: false,
      lit: snapshot.dnd,
      run: () => void commands.islandSetDnd(!snapshot.dnd),
    },
    {
      id: "focus",
      label: "dock.focus",
      text: focusing ? formatRemaining(pomodoro.status.remaining_secs) : undefined,
      active: panel === "focus",
      lit: focusing,
      run: () => onPanel(panel === "focus" ? null : "focus"),
    },
    {
      id: "tasks",
      label: "dock.tasks",
      active: panel === "tasks",
      run: () => onPanel(panel === "tasks" ? null : "tasks"),
    },
    {
      id: "net",
      label: "dock.net",
      active: panel === "net",
      run: () => onPanel(panel === "net" ? null : "net"),
    },
    {
      id: "ask",
      label: "dock.ask",
      active: kind === "ask" && !panel,
      run: () => {
        onPanel(null);
        void commands.askOpen();
      },
    },
    {
      id: "shelf",
      label: "dock.shelf",
      active: kind === "shelf" && !panel,
      run: () => {
        onPanel(null);
        void commands.shelfShow();
      },
    },
    {
      id: "clipboard",
      label: "dock.clipboard",
      active: panel === "clipboard",
      run: () => onPanel(panel === "clipboard" ? null : "clipboard"),
    },
    {
      id: "tools",
      label: "dock.tools",
      active: panel === "tools",
      run: () => onPanel(panel === "tools" ? null : "tools"),
    },
  ];

  // Satu label saja yang boleh melebar: hitung mundur Fokus menggantikan label tombol aktif supaya dok tidak meluber.
  const timerOn = items.some((it) => it.text !== undefined);

  return (
    <div className={calm ? "island-dock calm" : "island-dock"} onClick={(e) => e.stopPropagation()}>
      <div className="dock-tray">
        {items.map((it, i) => {
          const showLabel = (it.active && !timerOn) || it.text !== undefined;
          return (
            <button
              key={it.id}
              type="button"
              className={it.active ? "dock-btn active" : it.lit ? "dock-btn lit" : "dock-btn"}
              style={{ "--i": i } as CSSProperties}
              data-label={t(it.label)}
              aria-label={t(it.label)}
              aria-pressed={it.active || it.lit === true}
              disabled={it.disabled}
              onClick={it.run}
            >
              {it.active && (
                <motion.span
                  layoutId="dock-glow"
                  className="dock-glow"
                  transition={calm ? instant : glide}
                />
              )}
              <Icon name={it.id} />
              <AnimatePresence initial={false}>
                {showLabel && (
                  <motion.span
                    key="label"
                    className="dock-label"
                    initial={{ width: 0, opacity: 0 }}
                    animate={{ width: "auto", opacity: 1 }}
                    exit={{ width: 0, opacity: 0 }}
                    transition={calm ? instant : glide}
                  >
                    <span>{it.text ?? t(it.label)}</span>
                  </motion.span>
                )}
              </AnimatePresence>
            </button>
          );
        })}
        <span className="dock-sep" aria-hidden="true" />
        <button
          type="button"
          className="dock-btn dock-settings"
          style={{ "--i": items.length } as CSSProperties}
          data-label={t("dock.settings")}
          aria-label={t("dock.settings")}
          onClick={onOpenSettings}
        >
          <Icon name="settings" />
        </button>
      </div>
    </div>
  );
}
