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
import { IslandDock } from "./IslandDock";
import { IslandMenu, menuShape } from "./IslandMenu";
import { ShelfCard } from "../../shelf/view/ShelfCard";
import { ReviewCard } from "../../review/view/ReviewCard";
import { useSettingsStore } from "../../settings/store";
import { useIslandVM } from "../viewmodel/useIslandVM";

const panelShape = { width: 520, height: 298, radius: 28, opacity: 1 };

export function IslandShell() {
  const vm = useIslandVM();
  const t = useT();
  const ref = useRef<HTMLDivElement>(null);
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const reduceMotion = systemReduce || settingReduce;
  const { reportHitbox } = vm;

  // Lapor hitbox setiap ukuran berubah (selama animasi juga) agar klik tembus akurat.
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const observer = new ResizeObserver(() => reportHitbox(el.getBoundingClientRect()));
    observer.observe(el);
    return () => observer.disconnect();
  }, [reportHitbox]);

  const [menu, setMenu] = useState(false);
  const [panel, setPanel] = useState<"clipboard" | null>(null);
  const { content, mode } = vm;
  const expanded = mode === "expanded";
  const panelOpen = panel !== null && expanded;
  // Panel hanya hidup selama pulau terbuka.
  useEffect(() => {
    if (!expanded) setPanel(null);
  }, [expanded]);
  const shape = menu ? menuShape : panelOpen ? panelShape : vm.shape;
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
              <ClipboardPanel onClose={() => setPanel(null)} />
            ) : !content ? null : content.kind === "volume" ? (
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
