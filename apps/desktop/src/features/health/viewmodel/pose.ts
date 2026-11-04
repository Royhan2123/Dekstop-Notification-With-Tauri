import type { StretchMove } from "./useStretchVM";

/** Sudut sendi (radian) dan sedikit offset untuk satu karakter 3D; semua dihitung murni dari gerakan dan waktu. */
export interface Pose {
  bob: number; // naik-turun badan
  torsoX: number; // + membungkuk ke depan, - melengkung ke belakang
  torsoZ: number; // miring ke samping
  headX: number;
  headY: number;
  headZ: number;
  shrug: number; // tinggi bahu
  rollY: number; // bahu berputar (maju-mundur)
  armLX: number; // lengan atas kiri: ke depan (-) / belakang (+)
  armLZ: number; // lengan atas kiri: ke samping/atas
  armRX: number;
  armRZ: number;
  elbowL: number; // siku menekuk
  elbowR: number;
  wristL: number; // pergelangan berputar
  wristR: number;
  hand: number; // skala telapak (menggenggam/membuka)
}

export type PoseMode = StretchMove | "idle" | "done";

export const NEUTRAL: Pose = {
  bob: 0,
  torsoX: 0,
  torsoZ: 0,
  headX: 0,
  headY: 0,
  headZ: 0,
  shrug: 0,
  rollY: 0,
  armLX: 0,
  armLZ: 0.18,
  armRX: 0,
  armRZ: -0.18,
  elbowL: 0.1,
  elbowR: 0.1,
  wristL: 0,
  wristR: 0,
  hand: 1,
};

const TAU = Math.PI * 2;
const wave = (t: number, period: number, phase = 0) => Math.sin((t / period) * TAU + phase);

/** Pose target pada detik ke-`t` untuk gerakan tertentu; mulus berulang, tanpa lompatan. */
export function poseAt(mode: PoseMode, t: number): Pose {
  const breathe = wave(t, 3.2) * 0.012;
  const p: Pose = { ...NEUTRAL, bob: breathe };
  switch (mode) {
    case "idle":
      p.headZ = wave(t, 5) * 0.06;
      p.headY = wave(t, 7) * 0.1;
      p.armLZ = NEUTRAL.armLZ + wave(t, 3.2) * 0.03;
      p.armRZ = NEUTRAL.armRZ - wave(t, 3.2) * 0.03;
      break;
    case "neck": {
      const s = wave(t, 4);
      p.headZ = s * 0.5;
      p.headX = Math.abs(s) * 0.12;
      p.torsoZ = -s * 0.04;
      p.shrug = -Math.abs(s) * 0.03;
      break;
    }
    case "shoulders": {
      const s = wave(t, 2.6);
      const c = wave(t, 2.6, Math.PI / 2);
      p.shrug = Math.max(0, s) * 0.2 + 0.03;
      p.rollY = c * 0.28;
      p.headX = -Math.max(0, s) * 0.08;
      p.armLZ = 0.28 + Math.max(0, s) * 0.12;
      p.armRZ = -0.28 - Math.max(0, s) * 0.12;
      break;
    }
    case "wrists": {
      const s = wave(t, 2);
      p.armLX = -1.15;
      p.armRX = -1.15;
      p.armLZ = 0.22;
      p.armRZ = -0.22;
      p.elbowL = 0.9;
      p.elbowR = 0.9;
      p.wristL = (t / 2) * TAU;
      p.wristR = -(t / 2) * TAU;
      p.hand = 1 + s * 0.22;
      p.headX = 0.06;
      break;
    }
    case "back": {
      const s = wave(t, 4);
      p.armLZ = 2.7 + s * 0.08;
      p.armRZ = -2.7 - s * 0.08;
      p.elbowL = 0.08;
      p.elbowR = 0.08;
      p.torsoX = -0.16 - Math.max(0, s) * 0.1;
      p.torsoZ = wave(t, 8) * 0.12;
      p.headX = -0.2 - Math.max(0, s) * 0.12;
      p.bob = 0.03 + Math.max(0, s) * 0.02;
      p.hand = 1.1;
      break;
    }
    case "done": {
      const s = Math.abs(wave(t, 0.9));
      p.bob = s * 0.16;
      p.armLZ = 2.5 + wave(t, 0.9) * 0.2;
      p.armRZ = -2.5 - wave(t, 0.9) * 0.2;
      p.headZ = wave(t, 1.8) * 0.12;
      p.hand = 1.15;
      break;
    }
  }
  return p;
}

/** Dekati target secara eksponensial (peralihan antar-gerakan tanpa patah); `dt` detik, `rate` per detik. */
export function approach(cur: Pose, target: Pose, dt: number, rate = 7): Pose {
  const k = 1 - Math.exp(-rate * Math.max(0, dt));
  const out = { ...cur };
  for (const key of Object.keys(target) as (keyof Pose)[]) {
    out[key] = cur[key] + (target[key] - cur[key]) * k;
  }
  return out;
}
