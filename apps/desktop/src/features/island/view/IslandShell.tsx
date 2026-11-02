import { useEffect, useRef } from "react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useT } from "../../../shared/i18n";
import { instant, springs } from "../../../shared/motion/springs";
import { MediaCompact } from "../../media/view/MediaCompact";
import { MediaExpanded } from "../../media/view/MediaExpanded";
import { useSettingsStore } from "../../settings/store";
import { useIslandVM } from "../viewmodel/useIslandVM";

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

  const { content, mode, shape } = vm;
  return (
    <motion.div
      ref={ref}
      className="island"
      initial={false}
      animate={{
        width: shape.width,
        height: shape.height,
        borderRadius: shape.radius,
        opacity: shape.opacity,
      }}
      transition={reduceMotion ? instant : springs.island}
      onHoverStart={() => vm.onHover(true)}
      onHoverEnd={() => vm.onHover(false)}
      onClick={vm.onClick}
      onContextMenu={(e) => {
        e.preventDefault();
        vm.onOpenSettings();
      }}
    >
      <AnimatePresence mode="popLayout" initial={false}>
        {content && (
          <motion.div
            key={`${content.kind}-${mode}`}
            initial={{ opacity: 0, scale: 0.96 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.96 }}
            transition={reduceMotion ? instant : { duration: 0.15 }}
          >
            {content.kind === "media" ? (
              mode === "expanded" ? (
                <MediaExpanded />
              ) : (
                <MediaCompact />
              )
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
  );
}
