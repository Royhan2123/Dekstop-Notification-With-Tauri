import "./lumi.css";
import type { Mood } from "../../../shared/ipc";
import { useLumiVM } from "../viewmodel/useLumiVM";

const fill = (v: string) => ({ fill: `var(${v})` });
const EYE = fill("--m-eye");

/** Mata piksel di balik visor; tiap mata satu grup supaya bisa berkedip dan mengikuti kursor. */
function Eyes({ mood }: { mood: Mood }) {
  return (
    <>
      {[39, 61].map((x) => (
        <g className="lumi-eye" key={x}>
          <g className="lumi-pupil">
            {mood === "happy" ? (
              <path
                d={`M${x - 5} 62l5-7 5 7`}
                style={{ stroke: "var(--m-eye)" }}
                strokeWidth="3.4"
                fill="none"
                strokeLinejoin="round"
                strokeLinecap="square"
              />
            ) : mood === "sleepy" ? (
              <rect x={x - 5} y="58" width="10" height="3" rx="1" style={EYE} />
            ) : mood === "panic" ? (
              <>
                <rect x={x - 5} y="50" width="10" height="16" rx="1.5" style={EYE} />
                <rect x={x - 1.5} y="56" width="3" height="4" style={fill("--m-visor")} />
              </>
            ) : (
              <rect x={x - 4} y="52" width="8" height="12" rx="1.5" style={EYE} />
            )}
          </g>
        </g>
      ))}
    </>
  );
}

/** Hiasan kecil per ekspresi: titik berpikir, keringat, bintang, nada, Zzz. */
function Extras({ mood }: { mood: Mood }) {
  switch (mood) {
    case "thinking":
      return (
        <g style={fill("--m-soft")} className="lumi-dots">
          <circle cx="80" cy="16" r="2.4" />
          <circle cx="87" cy="11" r="2.4" />
          <circle cx="94" cy="6" r="2.4" />
        </g>
      );
    case "worried":
      return <path className="lumi-sweat" d="M84 28q5 8 0 11q-5-3 0-11z" fill="#7dd3fc" />;
    case "happy":
      return (
        <g fill="#ffd84d" className="lumi-sparkle">
          <path d="M86 10l2 4.4 4.4 2-4.4 2-2 4.4-2-4.4-4.4-2 4.4-2z" />
          <path d="M10 14l1.4 3 3 1.4-3 1.4-1.4 3-1.4-3-3-1.4 3-1.4z" />
        </g>
      );
    case "listening":
      return (
        <text className="lumi-note" x="82" y="22" fontSize="16" style={fill("--accent")}>
          ♪
        </text>
      );
    case "sleepy":
      return (
        <text className="lumi-zzz" x="82" y="22" fontSize="14" style={fill("--m-soft")}>
          z
        </text>
      );
    default:
      return null;
  }
}

/** Orbit cincin: bulan kecil. Hanya bergerak bila animasi diizinkan (diam saat idle). */
function Moon({ moving }: { moving: boolean }) {
  const body = (
    <>
      <circle r="4.6" fill="#ffd96a" stroke="#d9a93a" strokeWidth="1" />
      <circle cx="-1.4" cy="-1" r="1" fill="#d9a93a" />
    </>
  );
  return moving ? (
    <g>
      <animateMotion
        dur="9s"
        repeatCount="indefinite"
        path="M4 60A46 12 0 0 0 96 60A46 12 0 0 0 4 60Z"
      />
      {body}
    </g>
  ) : (
    <g transform="translate(24 69)">{body}</g>
  );
}

/** Mino: robot bulat mint dengan visor, mata piksel, dan cincin berbulan. SVG supaya ringan dan bisa dijeda total. */
export function LumiCanvas() {
  const vm = useLumiVM();
  const cls = ["lumi", `mood-${vm.mood}`, vm.calm && "calm", vm.poked && "poked"]
    .filter(Boolean)
    .join(" ");
  return (
    <svg
      ref={vm.ref}
      className={cls}
      viewBox="0 0 100 100"
      role="img"
      aria-label={`Mino (${vm.mood})`}
      onClick={(e) => {
        e.stopPropagation();
        vm.poke();
      }}
    >
      <defs>
        <radialGradient id="mino-body" cx="35%" cy="25%" r="90%">
          <stop offset="0" style={{ stopColor: "var(--m-hi)" }} />
          <stop offset=".55" style={{ stopColor: "var(--m-mid)" }} />
          <stop offset="1" style={{ stopColor: "var(--m-lo)" }} />
        </radialGradient>
      </defs>
      <ellipse cx="50" cy="95" rx="28" ry="3.4" fill="#000" opacity=".18" />
      <g className="lumi-bob">
        <g transform="rotate(-14 50 60)">
          <path
            d="M4 60A46 12 0 0 1 96 60"
            fill="none"
            style={{ stroke: "var(--accent)" }}
            strokeWidth="3"
            opacity=".9"
          />
        </g>
        <path d="M50 28V14" stroke="#7f9d95" strokeWidth="3" strokeLinecap="round" />
        <rect x="45.5" y="5" width="9" height="9" rx="2.5" fill="#ffb347" />
        <ellipse cx="50" cy="59" rx="34" ry="31" fill="url(#mino-body)" />
        <rect x="12" y="52" width="6" height="14" rx="3" style={fill("--accent")} />
        <rect x="82" y="52" width="6" height="14" rx="3" style={fill("--accent")} />
        <rect x="25" y="44" width="50" height="27" rx="9" style={fill("--m-visor")} />
        <path
          d="M31 49q19-4 38 0"
          stroke="#fff"
          opacity=".22"
          strokeWidth="2"
          fill="none"
          strokeLinecap="round"
        />
        <g transform="rotate(-14 50 60)">
          <path
            d="M4 60A46 12 0 0 0 96 60"
            fill="none"
            style={{ stroke: "var(--accent)" }}
            strokeWidth="3"
          />
          <Moon moving={!vm.calm} />
        </g>
        <ellipse
          cx="34"
          cy="38"
          rx="9"
          ry="4.5"
          transform="rotate(-25 34 38)"
          fill="#fff"
          opacity=".7"
        />
        <Eyes mood={vm.mood} />
      </g>
      <Extras mood={vm.mood} />
    </svg>
  );
}
