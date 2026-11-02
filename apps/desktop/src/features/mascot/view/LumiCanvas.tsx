import "./lumi.css";
import type { Mood } from "../../../shared/ipc";
import { useLumiVM } from "../viewmodel/useLumiVM";

function Eyes({ mood }: { mood: Mood }) {
  switch (mood) {
    case "happy":
      return <path className="lumi-line" d="M14 23 q4 -5 8 0 M26 23 q4 -5 8 0" />;
    case "sleepy":
      return <path className="lumi-line" d="M14 24 h8 M26 24 h8" />;
    case "panic":
      return (
        <g className="lumi-eyes">
          <circle cx="18" cy="23" r="5" fill="#fff" />
          <circle cx="30" cy="23" r="5" fill="#fff" />
          <circle cx="18" cy="23" r="1.6" fill="#111" />
          <circle cx="30" cy="23" r="1.6" fill="#111" />
        </g>
      );
    case "thinking":
      return (
        <g className="lumi-eyes lumi-look">
          <circle cx="18" cy="22" r="3.4" fill="#fff" />
          <circle cx="30" cy="22" r="3.4" fill="#fff" />
          <circle cx="19" cy="21" r="1.4" fill="#111" />
          <circle cx="31" cy="21" r="1.4" fill="#111" />
        </g>
      );
    default:
      return (
        <g className="lumi-eyes lumi-blink">
          <ellipse cx="18" cy="23" rx="3" ry="4" fill="#111" />
          <ellipse cx="30" cy="23" rx="3" ry="4" fill="#111" />
        </g>
      );
  }
}

function Mouth({ mood }: { mood: Mood }) {
  switch (mood) {
    case "happy":
      return <path className="lumi-line" d="M19 30 q5 6 10 0" />;
    case "worried":
      return <path className="lumi-line" d="M19 33 q5 -4 10 0" />;
    case "panic":
      return <ellipse cx="24" cy="33" rx="3" ry="4" fill="#111" />;
    case "sleepy":
      return <ellipse cx="24" cy="32" rx="2" ry="2.4" fill="#111" opacity="0.7" />;
    default:
      return <path className="lumi-line" d="M21 31 h6" />;
  }
}

/** Hiasan kecil per ekspresi: titik berpikir, keringat, bintang, nada, Zzz. */
function Extras({ mood }: { mood: Mood }) {
  switch (mood) {
    case "thinking":
      return (
        <g fill="#fff" className="lumi-dots">
          <circle cx="36" cy="8" r="1.6" />
          <circle cx="41" cy="6" r="1.6" />
          <circle cx="46" cy="4" r="1.6" />
        </g>
      );
    case "worried":
      return <path className="lumi-sweat" d="M37 14 q3 5 0 7 q-3 -2 0 -7z" fill="#7dd3fc" />;
    case "happy":
      return (
        <g fill="#ffd84d" className="lumi-sparkle">
          <path d="M6 8 l1.4 3 3 1.4 -3 1.4 -1.4 3 -1.4 -3 -3 -1.4 3 -1.4z" />
          <path d="M40 10 l1 2.2 2.2 1 -2.2 1 -1 2.2 -1 -2.2 -2.2 -1 2.2 -1z" />
        </g>
      );
    case "listening":
      return (
        <text className="lumi-note" x="36" y="12" fontSize="10" fill="#fff">
          ♪
        </text>
      );
    case "sleepy":
      return (
        <text className="lumi-zzz" x="34" y="12" fontSize="9" fill="#fff">
          z
        </text>
      );
    default:
      return null;
  }
}

/** Lumi: bola cahaya ungu-biru. Digambar sebagai SVG supaya ringan dan bisa dijeda total. */
export function LumiCanvas() {
  const vm = useLumiVM();
  const cls = ["lumi", `mood-${vm.mood}`, vm.calm && "calm", vm.poked && "poked"]
    .filter(Boolean)
    .join(" ");
  return (
    <svg
      className={cls}
      viewBox="0 0 48 48"
      role="img"
      aria-label={`Lumi (${vm.mood})`}
      onClick={(e) => {
        e.stopPropagation();
        vm.poke();
      }}
    >
      <defs>
        <radialGradient id="lumi-body" cx="35%" cy="30%" r="80%">
          <stop offset="0%" stopColor="#b9a4ff" />
          <stop offset="100%" stopColor="#4a8dff" />
        </radialGradient>
      </defs>
      <g className="lumi-bob">
        <ellipse
          className="lumi-body"
          cx="24"
          cy="26"
          rx="18"
          ry="16"
          fill={vm.mood === "panic" ? "#ff5d5d" : "url(#lumi-body)"}
        />
        <Eyes mood={vm.mood} />
        <Mouth mood={vm.mood} />
      </g>
      <Extras mood={vm.mood} />
    </svg>
  );
}
