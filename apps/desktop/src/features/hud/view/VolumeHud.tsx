import { useT } from "../../../shared/i18n";

interface Props {
  level: number;
  muted: boolean;
}

/**
 * HUD volume gaya pulau: ikon, bilah, dan persen; muncul sebentar saat volume berubah.
 * Data lewat props (bukan store) agar animasi keluar tetap menampilkan nilai terakhir HUD ini.
 */
export function VolumeHud({ level, muted }: Props) {
  const t = useT();

  return (
    <div
      className="volume-hud"
      role="status"
      aria-label={muted ? t("hud.muted") : `${t("island.volume")} ${level}%`}
    >
      <span className="volume-icon" aria-hidden="true">
        {muted || level === 0 ? "🔇" : level < 50 ? "🔉" : "🔊"}
      </span>
      <div className="volume-bar">
        <div className="volume-bar-fill" style={{ width: `${muted ? 0 : level}%` }} />
      </div>
      <span className="volume-value">{muted ? t("hud.muted") : `${level}%`}</span>
    </div>
  );
}
