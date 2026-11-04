import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useT } from "../../../shared/i18n";
import { instant, springs } from "../../../shared/motion/springs";
import { VolumeHud } from "../../hud/view/VolumeHud";
import { AskCard } from "../../ask/view/AskCard";
import { LumiCanvas } from "../../mascot/view/LumiCanvas";
import { MediaCompact } from "../../media/view/MediaCompact";
import { MediaExpanded } from "../../media/view/MediaExpanded";
import { ClipboardPanel } from "../../clipboard/view/ClipboardPanel";
import { HealthPanel } from "../../health/view/HealthPanel";
import { NetPanel } from "../../net/view/NetPanel";
import { FocusPanel } from "../../pomodoro/view/FocusPanel";
import { TasksPanel } from "../../tasks/view/TasksPanel";
import { ToolsPanel } from "../../tools/view/ToolsPanel";
import { useReminders } from "../../health/viewmodel/useReminders";
import { IslandDock, type Panel } from "./IslandDock";
import { IslandMenu, menuShape } from "./IslandMenu";
import { ShelfCard } from "../../shelf/view/ShelfCard";
import { ReviewCard } from "../../review/view/ReviewCard";
import { useSettingsStore } from "../../settings/store";
import { useDynamicBackground } from "../../theme/viewmodel/useDynamicBackground";
import { EXPANDED_W, useIslandVM } from "../viewmodel/useIslandVM";

const panelShapes = {
  clipboard: { width: EXPANDED_W, height: 258, radius: 26, opacity: 1 },
  health: { width: EXPANDED_W, height: 290, radius: 26, opacity: 1 },
  net: { width: EXPANDED_W, height: 218, radius: 26, opacity: 1 },
  focus: { width: EXPANDED_W, height: 228, radius: 26, opacity: 1 },
  tasks: { width: EXPANDED_W, height: 308, radius: 26, opacity: 1 },
  tools: { width: EXPANDED_W, height: 340, radius: 26, opacity: 1 },
};

/** Selama animasi ukuran, hitbox dilaporkan paling sering sekali per jeda ini (bukan tiap frame). */
const HITBOX_MS = 50;
/** Sisa ruang di tepi jendela bila layar lebih sempit dari lebar kartu. */
const EDGE = 16;

function useViewportWidth() {
  const [w, setW] = useState(() => window.innerWidth);
  useEffect(() => {
    const on = () => setW(window.innerWidth);
    window.addEventListener("resize", on);
    return () => window.removeEventListener("resize", on);
  }, []);
  return w;
}

export function IslandShell() {
  const vm = useIslandVM();
  useDynamicBackground();
  const t = useT();
  const ref = useRef<HTMLDivElement>(null);
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const reduceMotion = systemReduce || settingReduce;
  const { reportHitbox } = vm;

  // Lapor hitbox setiap ukuran berubah (selama animasi juga, dijarangkan) agar klik tembus akurat.
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    let timer: number | undefined;
    let last = 0;
    const send = () => {
      timer = undefined;
      last = performance.now();
      reportHitbox(el.getBoundingClientRect());
    };
    const observer = new ResizeObserver(() => {
      if (timer === undefined)
        timer = window.setTimeout(send, Math.max(0, HITBOX_MS - (performance.now() - last)));
    });
    observer.observe(el);
    return () => {
      observer.disconnect();
      window.clearTimeout(timer);
    };
  }, [reportHitbox]);
  const viewport = useViewportWidth();
  useReminders();

  const [menu, setMenu] = useState(false);
  const [panel, setPanel] = useState<Panel>(null);
  const { content, mode } = vm;
  const expanded = mode === "expanded";
  const panelOpen = panel !== null && expanded;
  // Panel hanya lapisan di depan kartu backend; menutupnya juga menutup kartu buatan pengguna di belakangnya
  // (Rak, Tanya AI, Musik) supaya pulau tidak "berpindah" ke kartu itu. Hasil review tetap menunggu.
  const closePanel = () => {
    setPanel(null);
    if (content && ["ask", "shelf", "media"].includes(content.kind)) vm.onDismiss();
  };
  // Panel hanya hidup selama pulau terbuka.
  useEffect(() => {
    if (!expanded) setPanel(null);
  }, [expanded]);
  const base = menu ? menuShape : panelOpen && panel ? panelShapes[panel] : vm.shape;
  const shape = { ...base, width: Math.min(base.width, Math.max(0, viewport - EDGE)) };
  return (
    <motion.div
      ref={ref}
      className="island"
      data-mode={mode}
      initial={false}
      animate={{
        width: shape.width,
        height: shape.height,
        borderRadius: shape.radius,
        opacity: shape.opacity,
      }}
      transition={reduceMotion ? instant : springs.island}
      onHoverStart={() => vm.onHover(true)}
      onHoverEnd={() => {
        setMenu(false);
        vm.onHover(false);
      }}
      onClick={vm.onClick}
      onContextMenu={(e) => {
        e.preventDefault();
        setMenu(true);
      }}
    >
      <AnimatePresence mode="popLayout" initial={false}>
        {(menu || panelOpen || content) && (
          <motion.div
            key={menu ? "menu" : panelOpen ? "panel" : `${content?.kind}-${mode}`}
            className={menu ? "island-content no-gutter" : "island-content"}
            initial={{ opacity: 0, scale: 0.96 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.96 }}
            transition={reduceMotion ? instant : { duration: 0.15 }}
          >
            {menu ? (
              <IslandMenu onClose={() => setMenu(false)} onOpenSettings={vm.onOpenSettings} />
            ) : panelOpen ? (
              panel === "health" ? (
                <HealthPanel onClose={closePanel} />
              ) : panel === "focus" ? (
                <FocusPanel onClose={closePanel} />
              ) : panel === "net" ? (
                <NetPanel onClose={closePanel} />
              ) : panel === "tasks" ? (
                <TasksPanel onClose={closePanel} />
              ) : panel === "tools" ? (
                <ToolsPanel onClose={closePanel} />
              ) : (
                <ClipboardPanel onClose={closePanel} />
              )
            ) : !content ? null : content.kind === "brightness" ? (
              <VolumeHud level={Number(content.text) || 0} muted={false} kind="brightness" />
            ) : content.kind === "volume" ? (
              <VolumeHud level={Number(content.text) || 0} muted={content.detail === "muted"} />
            ) : content.kind === "shelf" && mode === "expanded" ? (
              <ShelfCard />
            ) : content.kind === "ask" && mode === "expanded" ? (
              <AskCard />
            ) : content.kind === "media" ? (
              mode === "expanded" ? (
                <MediaExpanded />
              ) : (
                <MediaCompact />
              )
            ) : content.kind === "review_result" && vm.hasFindings && mode === "expanded" ? (
              <ReviewCard />
            ) : (
              <div className={mode === "expanded" ? "island-body expanded" : "island-body"}>
                <span className="island-label">{t(`island.${content.kind}`)}</span>
                {content.text && <span className="island-text">{content.text}</span>}
                {mode === "expanded" && (
                  <button
                    type="button"
                    className="island-dismiss"
                    onClick={(e) => {
                      e.stopPropagation();
                      vm.onDismiss();
                    }}
                  >
                    {t("island.dismiss")}
                  </button>
                )}
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
      {expanded && !menu && (
        <IslandDock panel={panel} onPanel={setPanel} onOpenSettings={vm.onOpenSettings} />
      )}
      {mode !== "hidden" && !menu && <LumiCanvas />}
    </motion.div>
  );
}
