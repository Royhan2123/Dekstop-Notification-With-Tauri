import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useT } from "../../../shared/i18n";
import { instant, springs } from "../../../shared/motion/springs";
import { VolumeHud } from "../../hud/view/VolumeHud";
import { AskCard } from "../../ask/view/AskCard";
import { LumiCanvas } from "../../mascot/view/LumiCanvas";
import { MediaCompact } from "../../media/view/MediaCompact";
import { MediaExpanded } from "../../media/view/MediaExpanded";
import { IslandMenu, menuShape } from "./IslandMenu";
import { ShelfCard } from "../../shelf/view/ShelfCard";
import { ReviewCard } from "../../review/view/ReviewCard";
import { useSettingsStore } from "../../settings/store";
import { useIslandVM } from "../viewmodel/useIslandVM";

/** Seberapa jauh Mino menjorok ke kiri dari tepi pulau per mode (px); samakan dengan lumi.css. */
const MINO_REACH: Record<string, number> = { idle: 36, compact: 48, expanded: 76 };

export function IslandShell() {
  const vm = useIslandVM();
  const t = useT();
  const ref = useRef<HTMLDivElement>(null);
  const systemReduce = useReducedMotion();
  const settingReduce = useSettingsStore((s) => s.settings?.reduce_motion === true);
  const reduceMotion = systemReduce || settingReduce;
  const { reportHitbox } = vm;
  const modeRef = useRef(vm.mode);
  modeRef.current = vm.mode;

  // Lapor hitbox setiap ukuran berubah (selama animasi juga) agar klik tembus akurat.
  // Mino duduk di luar tepi kiri pulau, jadi hitbox diperluas ke kiri sejauh MINO_REACH.
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const observer = new ResizeObserver(() => {
      const r = el.getBoundingClientRect();
      const reach = MINO_REACH[modeRef.current] ?? 0;
      reportHitbox(new DOMRect(r.x - reach, r.y, r.width + reach, r.height));
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, [reportHitbox]);

  const [menu, setMenu] = useState(false);
  const { content, mode } = vm;
  const shape = menu ? menuShape : vm.shape;
  return (
    <div className="island-anchor" data-mode={mode}>
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
          {(menu || content) && (
            <motion.div
              key={menu ? "menu" : `${content?.kind}-${mode}`}
              initial={{ opacity: 0, scale: 0.96 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.96 }}
              transition={reduceMotion ? instant : { duration: 0.15 }}
            >
              {menu ? (
                <IslandMenu onClose={() => setMenu(false)} onOpenSettings={vm.onOpenSettings} />
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
      </motion.div>
      {mode !== "hidden" && !menu && <LumiCanvas />}
    </div>
  );
}
