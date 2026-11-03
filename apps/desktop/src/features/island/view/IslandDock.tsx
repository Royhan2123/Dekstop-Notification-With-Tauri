import { useRef, useState } from "react";
import { useT, type MessageKey } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { formatRemaining, usePomodoroVM } from "../../pomodoro/viewmodel/usePomodoroVM";
import { useIslandStore } from "../store";

export type Panel = "clipboard" | "health" | "net" | "tasks" | "focus" | null;

const ICONS = {
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
      width="16"
      height="16"
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

/** Tombol per halaman carousel dok. */
const PER_PAGE = 3;

interface Item {
  id: keyof typeof ICONS;
  label: MessageKey;
  /** Teks di tombol bila bukan label (mis. hitung mundur). */
  text?: string;
  active: boolean;
  run: () => void;
}

/**
 * Dok fitur di dasar pulau yang terbuka: carousel per halaman (3 tombol) yang bisa digeser dengan
 * seret, roda mouse, atau titik indikator, plus tombol Setelan yang selalu terlihat di kanan.
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
  const drag = useRef({ x: 0, page: 0, moved: false, down: false });
  const wheelLock = useRef(false);
  const [page, setPage] = useState(0);
  const kind = snapshot.content?.kind;
  const focusing = pomodoro.status.phase !== "idle";

  const items: Item[] = [
    {
      id: "health",
      label: "dock.health",
      active: panel === "health",
      run: () => onPanel(panel === "health" ? null : "health"),
    },
    {
      id: "dnd",
      label: "dock.dnd",
      active: snapshot.dnd,
      run: () => void commands.islandSetDnd(!snapshot.dnd),
    },
    {
      id: "focus",
      label: "dock.focus",
      text: focusing ? formatRemaining(pomodoro.status.remaining_secs) : undefined,
      active: focusing || panel === "focus",
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
  ];

  const pages = Array.from({ length: Math.ceil(items.length / PER_PAGE) }, (_, i) =>
    items.slice(i * PER_PAGE, (i + 1) * PER_PAGE),
  );
  const goTo = (p: number) => {
    const el = scroller.current;
    if (!el) return;
    const target = Math.max(0, Math.min(pages.length - 1, p));
    el.scrollTo({ left: target * el.clientWidth, behavior: "smooth" });
  };

  return (
    <div className="island-dock" onClick={(e) => e.stopPropagation()}>
      <div className="dock-main">
        <div
          ref={scroller}
          className="dock-scroll"
          onScroll={(e) => {
            const el = e.currentTarget;
            setPage(Math.round(el.scrollLeft / Math.max(1, el.clientWidth)));
          }}
          onWheel={(e) => {
            if (wheelLock.current) return;
            wheelLock.current = true;
            window.setTimeout(() => (wheelLock.current = false), 350);
            goTo(page + (e.deltaY + e.deltaX > 0 ? 1 : -1));
          }}
          onPointerDown={(e) => {
            drag.current = { x: e.clientX, page, moved: false, down: true };
          }}
          onPointerMove={(e) => {
            const d = drag.current;
            const el = scroller.current;
            if (!d.down || !el) return;
            const dx = e.clientX - d.x;
            if (Math.abs(dx) > 4) {
              d.moved = true;
              el.style.scrollSnapType = "none";
            }
            if (d.moved) el.scrollLeft = d.page * el.clientWidth - dx;
          }}
          onPointerUp={(e) => {
            const d = drag.current;
            if (!d.down) return;
            d.down = false;
            if (!d.moved || !scroller.current) return;
            scroller.current.style.scrollSnapType = "";
            const dx = e.clientX - d.x;
            goTo(d.page + (dx < -40 ? 1 : dx > 40 ? -1 : 0));
          }}
          onPointerLeave={() => {
            if (!drag.current.down) return;
            drag.current.down = false;
            if (scroller.current) scroller.current.style.scrollSnapType = "";
            goTo(page);
          }}
          // Seretan tidak boleh dihitung sebagai klik pada tombol.
          onClickCapture={(e) => {
            if (drag.current.moved) {
              e.stopPropagation();
              drag.current.moved = false;
            }
          }}
        >
          {pages.map((p, i) => (
            <div className="dock-page" key={i}>
              {p.map((it) => (
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
                  <span>{it.text ?? t(it.label)}</span>
                </button>
              ))}
            </div>
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
      <div className="dock-dots">
        {pages.map((_, i) => (
          <button
            key={i}
            type="button"
            className={i === page ? "dot active" : "dot"}
            aria-label={`${i + 1}/${pages.length}`}
            onClick={() => goTo(i)}
          />
        ))}
      </div>
    </div>
  );
}
