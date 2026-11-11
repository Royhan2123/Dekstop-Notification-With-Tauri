import { useT } from "../../../shared/i18n";

interface Props {
  /** Fase Jarvis: `listening`, `thinking`, atau `speaking`. */
  phase: string;
}

const BARS = [0, 1, 2, 3, 4];

/**
 * Indikator asisten suara Jarvis di pulau: lima batang yang bergerak menurut fase, ditambah label.
 * Gerak murni CSS (lihat `.voice-bars`) sehingga ringan dan ikut mati bila gerak dikurangi.
 */
export function VoiceBars({ phase }: Props) {
  const t = useT();
  const label =
    phase === "thinking"
      ? t("voice.thinking")
      : phase === "speaking"
        ? t("voice.speaking")
        : t("voice.listening");
  return (
    <div className="voice-hud" role="status" aria-label={label}>
      <div className="voice-bars" data-phase={phase} aria-hidden="true">
        {BARS.map((i) => (
          <span key={i} style={{ animationDelay: `${i * 0.11}s` }} />
        ))}
      </div>
      <span className="voice-label">{label}</span>
    </div>
  );
}
