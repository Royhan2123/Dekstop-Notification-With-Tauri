import "./lumi.css";
import type { Mood } from "../../../shared/ipc";
import { useLumiVM } from "../viewmodel/useLumiVM";

/** Bola mata putih + pupil bersorot; pupil bergeser mengikuti kursor lewat variabel CSS --lx/--ly. */
function Eye({ cx, cy, r = 3 }: { cx: number; cy: number; r?: number }) {
  return (
    <g className="lumi-eye">
      <ellipse cx={cx} cy={cy} rx="4.8" ry="5.6" fill="#fff" />
      <g className="lumi-pupil">
        <circle cx={cx} cy={cy + 0.4} r={r} fill="#1b1840" />
        <circle cx={cx - 1} cy={cy - 1} r="1.1" fill="#fff" />
        <circle cx={cx + 1.1} cy={cy + 1.6} r="0.5" fill="#fff" opacity="0.7" />
      </g>
    </g>
  );
}

function Eyes({ mood }: { mood: Mood }) {
  switch (mood) {
    case "happy":
      return <path className="lumi-line" d="M13 24 q4 -5 8 0 M27 24 q4 -5 8 0" />;
    case "sleepy":
      return <path className="lumi-line" d="M13 24 h8 M27 24 h8" />;
    case "panic":
      return (
        <>
          <Eye cx={17} cy={23} r={1.6} />
          <Eye cx={31} cy={23} r={1.6} />
        </>
      );
    default:
      return (
        <>
          <Eye cx={17} cy={23} />
          <Eye cx={31} cy={23} />
        </>
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

/** Lumi: bola cahaya ungu-biru bergaya 3D (gradasi, kilau, bayangan). SVG supaya ringan dan bisa dijeda total. */
export function LumiCanvas() {
  const vm = useLumiVM();
  const cls = ["lumi", `mood-${vm.mood}`, vm.calm && "calm", vm.poked && "poked"]
    .filter(Boolean)
    .join(" ");
  return (
    <svg
      ref={vm.ref}
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
        <radialGradient id="lumi-body" cx="35%" cy="28%" r="80%">
          <stop offset="0%" stopColor="#e2d8ff" />
          <stop offset="45%" stopColor="#9a86ff" />
          <stop offset="100%" stopColor="#4262f0" />
        </radialGradient>
        <linearGradient id="lumi-shade" x1="0" y1="0" x2="0" y2="1">
          <stop offset="55%" stopColor="#1a1a6a" stopOpacity="0" />
          <stop offset="100%" stopColor="#1a1a6a" stopOpacity="0.4" />
        </linearGradient>
      </defs>
      <ellipse cx="24" cy="44.5" rx="13" ry="2.4" fill="#000" opacity="0.28" />
      <g className="lumi-bob">
        <path className="lumi-antenna" d="M24 9 q-1.5 -4 2.5 -6" fill="none" stroke="#b9a8ff" strokeWidth="1.6" strokeLinecap="round" />
        <circle cx="27" cy="3.4" r="2.2" fill="#ffd84d" />
        <ellipse
          className="lumi-body"
          cx="24"
          cy="26"
          rx="19"
          ry="17"
          fill={vm.mood === "panic" ? "#ff5d5d" : "url(#lumi-body)"}
        />
        <ellipse cx="24" cy="26" rx="19" ry="17" fill="url(#lumi-shade)" />
        <ellipse cx="15" cy="15.5" rx="6" ry="3.4" transform="rotate(-28 15 15.5)" fill="#fff" opacity="0.55" />
        <circle cx="9.5" cy="22" r="1.1" fill="#fff" opacity="0.55" />
        <ellipse cx="10.5" cy="31" rx="3.2" ry="2" fill="#ff8fb1" opacity="0.55" />
        <ellipse cx="37.5" cy="31" rx="3.2" ry="2" fill="#ff8fb1" opacity="0.55" />
        <Eyes mood={vm.mood} />
        <Mouth mood={vm.mood} />
      </g>
      <Extras mood={vm.mood} />
    </svg>
  );
}
