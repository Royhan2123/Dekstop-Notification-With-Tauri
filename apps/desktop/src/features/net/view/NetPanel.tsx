import { useT } from "../../../shared/i18n";
import { HISTORY, formatRate, sparkline, useNetVM } from "../viewmodel/useNetVM";

/** Kecepatan unduh dan unggah saat ini, plus grafik mini 30 detik terakhir. */
export function NetPanel({ onClose }: { onClose: () => void }) {
  const t = useT();
  const net = useNetVM();
  const now = (h: number[]) => h[h.length - 1] ?? 0;
  const bars = sparkline(net.down);

  return (
    <div className="net-panel" onClick={(e) => e.stopPropagation()}>
      <div className="shelf-head">
        <span>{t("dock.net")}</span>
        <button type="button" aria-label={t("island.dismiss")} onClick={onClose}>
          ×
        </button>
      </div>
      {net.error ? (
        <p className="shelf-empty">{net.error}</p>
      ) : (
        <>
          <div className="net-rates">
            <span className="net-down">↓ {formatRate(now(net.down))}</span>
            <span className="net-up">↑ {formatRate(now(net.up))}</span>
          </div>
          <div className="net-spark" role="img" aria-label={t("dock.net")}>
            {Array.from({ length: HISTORY }, (_, i) => {
              const pct = bars[i - (HISTORY - bars.length)] ?? 0;
              return <i key={i} style={{ height: `${Math.max(pct, 3)}%` }} />;
            })}
          </div>
        </>
      )}
    </div>
  );
}
