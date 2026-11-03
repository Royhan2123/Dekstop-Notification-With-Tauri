import { useRef } from "react";
import { useT, type MessageKey } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { usePomodoroVM } from "../../pomodoro/viewmodel/usePomodoroVM";
import { useIslandStore } from "../store";

type Panel = "clipboard" | null;

const ICONS = {
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
      width="19"
      height="19"
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

interface Item {
  id: keyof typeof ICONS;
  label: MessageKey;
  active: boolean;
  run: () => void;
}

/**
 * Dok fitur di dasar pulau yang terbuka: deretan ikon yang bisa digeser (seret, roda mouse, atau
 * sentuh), plus tombol Setelan yang selalu terlihat di kanan.
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
  const scroller = useRef<HTMLDivElement>(null);
  const drag = useRef({ x: 0, left: 0, moved: false, down: false });
  const kind = snapshot.content?.kind;
  const focusing = pomodoro.status.phase !== "idle";

  const items: Item[] = [
    {
      id: "dnd",
      label: "dock.dnd",
      active: snapshot.dnd,
      run: () => void commands.islandSetDnd(!snapshot.dnd),
    },
    {
      id: "focus",
      label: focusing ? "dock.focus_stop" : "dock.focus",
      active: focusing,
      run: focusing ? pomodoro.stop : pomodoro.start,
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
  ];

  return (
    <div className="island-dock" onClick={(e) => e.stopPropagation()}>
      <div
        ref={scroller}
        className="dock-scroll"
        onWheel={(e) => {
          if (scroller.current) scroller.current.scrollLeft += e.deltaY + e.deltaX;
        }}
        onPointerDown={(e) => {
          drag.current = {
            x: e.clientX,
            left: scroller.current?.scrollLeft ?? 0,
            moved: false,
            down: true,
          };
        }}
        onPointerMove={(e) => {
          const d = drag.current;
          if (!d.down || !scroller.current) return;
          const dx = e.clientX - d.x;
          if (Math.abs(dx) > 4) d.moved = true;
          scroller.current.scrollLeft = d.left - dx;
        }}
        onPointerUp={() => {
          drag.current.down = false;
        }}
        onPointerLeave={() => {
          drag.current.down = false;
        }}
        // Seretan tidak boleh dihitung sebagai klik pada ikon.
        onClickCapture={(e) => {
          if (drag.current.moved) {
            e.stopPropagation();
            drag.current.moved = false;
          }
        }}
      >
        {items.map((it) => (
          <button
            key={it.id}
            type="button"
            className={it.active ? "dock-btn active" : "dock-btn"}
            title={t(it.label)}
            aria-label={t(it.label)}
            aria-pressed={it.active}
            onClick={it.run}
          >
            <Icon name={it.id} />
          </button>
        ))}
      </div>
      <button
        type="button"
        className="dock-btn dock-settings"
        title={t("dock.settings")}
        aria-label={t("dock.settings")}
        onClick={onOpenSettings}
      >
        <Icon name="settings" />
      </button>
    </div>
  );
}
