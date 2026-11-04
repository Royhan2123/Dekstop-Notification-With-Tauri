import type * as THREE_NS from "three";
import type { AvatarSpec } from "../viewmodel/avatars";
import { NEUTRAL, type Pose } from "../viewmodel/pose";

type T = typeof THREE_NS;

export interface Rig {
  scene: THREE_NS.Scene;
  apply(pose: Pose, t: number): void;
  dispose(): void;
}

/**
 * Karakter 3D prosedural (tanpa berkas model): badan "tanah liat" dari bola dan kapsul bercahaya lembut.
 * Satu kerangka sendi dipakai semua karakter; AvatarSpec hanya mengubah warna, telinga, dan pernak-pernik.
 */
export function buildAvatar(THREE: T, spec: AvatarSpec): Rig {
  const scene = new THREE.Scene();
  const mats: THREE_NS.Material[] = [];
  const mat = (color: number, extra: THREE_NS.MeshStandardMaterialParameters = {}) => {
    const m = new THREE.MeshStandardMaterial({ color, roughness: 0.55, metalness: 0.04, ...extra });
    mats.push(m);
    return m;
  };
  const mesh = (g: THREE_NS.BufferGeometry, m: THREE_NS.Material) => new THREE.Mesh(g, m);
  const sphere = (r: number, m: THREE_NS.Material, sx = 1, sy = 1, sz = 1) => {
    const s = mesh(new THREE.SphereGeometry(r, 32, 24), m);
    s.scale.set(sx, sy, sz);
    return s;
  };
  const capsule = (r: number, len: number, m: THREE_NS.Material) =>
    mesh(new THREE.CapsuleGeometry(r, len, 8, 20), m);
  const at = <O extends THREE_NS.Object3D>(o: O, x: number, y: number, z: number) => {
    o.position.set(x, y, z);
    return o;
  };

  const skin = mat(spec.skin);
  const body = mat(spec.body);
  const accent = mat(spec.accent, { roughness: 0.4 });
  const eyeMat = mat(
    spec.eye,
    spec.faceplate ? { emissive: spec.eye, emissiveIntensity: 1.2 } : {},
  );
  const dark = mat(0x1b1f2e, { roughness: 0.25, metalness: 0.3 });
  const white = mat(0xffffff, { roughness: 0.3 });

  // Cahaya: langit lembut, cahaya utama dari kiri-atas, dan cahaya tepi dari belakang untuk kesan 3D.
  scene.add(new THREE.HemisphereLight(0xffffff, 0x8a96c8, 1.15));
  const key = new THREE.DirectionalLight(0xffffff, 2.1);
  key.position.set(2.5, 4, 4.5);
  scene.add(key);
  const rim = new THREE.DirectionalLight(spec.accent, 1.1);
  rim.position.set(-3, 2.5, -3);
  scene.add(rim);

  const root = new THREE.Group();
  scene.add(root);

  // Bayangan lembut di lantai.
  const shadowCanvas = document.createElement("canvas");
  shadowCanvas.width = shadowCanvas.height = 64;
  const sg = shadowCanvas.getContext("2d");
  if (sg) {
    const grad = sg.createRadialGradient(32, 32, 2, 32, 32, 32);
    grad.addColorStop(0, "rgba(0,0,0,0.5)");
    grad.addColorStop(1, "rgba(0,0,0,0)");
    sg.fillStyle = grad;
    sg.fillRect(0, 0, 64, 64);
  }
  const shadowTex = new THREE.CanvasTexture(shadowCanvas);
  const shadowMat = new THREE.MeshBasicMaterial({
    map: shadowTex,
    transparent: true,
    depthWrite: false,
  });
  mats.push(shadowMat);
  const shadow = mesh(new THREE.CircleGeometry(0.85, 32), shadowMat);
  shadow.rotation.x = -Math.PI / 2;
  shadow.position.y = 0.01;
  scene.add(shadow);

  // Kaki.
  for (const x of [-0.16, 0.16]) {
    root.add(at(capsule(0.12, 0.18, body), x, 0.26, 0));
    root.add(at(sphere(0.15, accent, 1, 0.62, 1.35), x, 0.1, 0.05));
  }

  // Badan (poros di pinggul).
  const torso = new THREE.Group();
  torso.position.y = 0.42;
  root.add(torso);
  torso.add(at(capsule(0.27, 0.2, body), 0, 0.37, 0));
  torso.add(at(sphere(0.17, white, 1, 1.15, 0.45), 0, 0.36, 0.22));
  if (spec.extra === "helmet")
    torso.add(at(mesh(new THREE.BoxGeometry(0.46, 0.56, 0.22), accent), 0, 0.4, -0.3));

  // Lengan: bahu -> lengan atas -> siku -> lengan bawah -> telapak.
  const makeArm = (side: 1 | -1) => {
    const shoulder = at(new THREE.Group(), side * 0.34, 0.58, 0);
    const upper = new THREE.Group();
    shoulder.add(upper);
    upper.add(at(capsule(0.09, 0.2, body), 0, -0.17, 0));
    const elbow = at(new THREE.Group(), 0, -0.34, 0);
    upper.add(elbow);
    elbow.add(at(capsule(0.085, 0.16, body), 0, -0.14, 0));
    const wrist = at(new THREE.Group(), 0, -0.3, 0);
    elbow.add(wrist);
    const hand = new THREE.Group();
    wrist.add(hand);
    hand.add(sphere(0.115, skin));
    hand.add(at(sphere(0.05, skin), side * 0.1, 0.02, 0.04)); // ibu jari: putaran pergelangan jadi terlihat
    torso.add(shoulder);
    return { shoulder, upper, elbow, wrist, hand, side };
  };
  const armL = makeArm(1);
  const armR = makeArm(-1);

  // Kepala.
  const head = at(new THREE.Group(), 0, 0.72, 0);
  torso.add(head);
  head.add(at(sphere(0.5, skin, 1.04, 0.94, 0.98), 0, 0.42, 0));
  const eyes: THREE_NS.Mesh[] = [];
  if (spec.faceplate) head.add(at(sphere(0.42, dark, 1, 0.62, 0.55), 0, 0.44, 0.3));
  for (const x of [-0.17, 0.17]) {
    const big = spec.id === "alien";
    const e = at(
      sphere(big ? 0.1 : 0.075, eyeMat, 1, big ? 1.5 : 1.15, 0.6),
      x * (big ? 1.25 : 1),
      0.47,
      big ? 0.46 : spec.faceplate ? 0.5 : 0.45,
    );
    head.add(e);
    eyes.push(e);
    if (!spec.faceplate)
      head.add(at(sphere(0.022, white), x * (big ? 1.25 : 1) + 0.025, 0.5, 0.51));
  }
  if (!spec.faceplate) {
    const smile = mesh(new THREE.TorusGeometry(0.06, 0.013, 8, 20, Math.PI), dark);
    smile.rotation.z = Math.PI;
    head.add(at(smile, 0, 0.36, 0.48));
  }
  if (spec.ears === "cat" || spec.ears === "bear") {
    for (const x of [-0.31, 0.31])
      head.add(at(sphere(0.07, mat(0xff8fb1), 1.4, 0.7, 0.6), x * 0.92, 0.28, 0.43)); // pipi
  }
  if (spec.ears === "cat") {
    for (const s of [-1, 1] as const) {
      const ear = at(mesh(new THREE.ConeGeometry(0.17, 0.32, 4), skin), s * 0.3, 0.83, 0);
      ear.rotation.z = -s * 0.35;
      head.add(ear);
      const inner = at(
        mesh(new THREE.ConeGeometry(0.09, 0.2, 4), mat(0xff8fb1)),
        s * 0.3,
        0.8,
        0.05,
      );
      inner.rotation.z = -s * 0.35;
      head.add(inner);
    }
  }
  if (spec.ears === "bear") {
    for (const s of [-1, 1] as const) {
      head.add(at(sphere(0.17, skin), s * 0.34, 0.84, -0.04));
      head.add(at(sphere(0.09, accent, 1, 1, 0.5), s * 0.34, 0.84, 0.08));
    }
    head.add(at(sphere(0.19, accent, 1.2, 0.85, 0.7), 0, 0.33, 0.4)); // moncong
    head.add(at(sphere(0.045, dark, 1.2, 0.8, 0.8), 0, 0.4, 0.55));
  }
  if (spec.extra === "antenna") {
    head.add(at(mesh(new THREE.CylinderGeometry(0.018, 0.018, 0.22, 8), dark), 0, 0.97, 0));
    head.add(at(sphere(0.07, accent), 0, 1.1, 0));
  }
  if (spec.extra === "antennae") {
    for (const s of [-1, 1] as const) {
      const stalk = at(
        mesh(new THREE.CylinderGeometry(0.016, 0.016, 0.3, 8), skin),
        s * 0.14,
        0.99,
        0,
      );
      stalk.rotation.z = -s * 0.35;
      head.add(stalk);
      head.add(at(sphere(0.06, accent), s * 0.2, 1.13, 0));
    }
  }
  if (spec.extra === "helmet") {
    const glass = mat(0xbfe3ff, {
      transparent: true,
      opacity: 0.2,
      roughness: 0.05,
      metalness: 0.1,
    });
    glass.depthWrite = false;
    head.add(at(sphere(0.66, glass), 0, 0.42, 0));
    const ring = mesh(new THREE.TorusGeometry(0.4, 0.07, 12, 32), accent);
    ring.rotation.x = Math.PI / 2;
    head.add(at(ring, 0, -0.02, 0));
  }

  const apply = (p: Pose, t: number) => {
    root.position.y = p.bob;
    shadow.scale.setScalar(1 - p.bob * 0.6);
    torso.rotation.set(p.torsoX, 0, p.torsoZ);
    head.rotation.set(p.headX, p.headY, p.headZ);
    for (const [arm, ax, az, el, wr] of [
      [armL, p.armLX, p.armLZ, p.elbowL, p.wristL],
      [armR, p.armRX, p.armRZ, p.elbowR, p.wristR],
    ] as const) {
      arm.shoulder.position.set(arm.side * 0.34, 0.58 + p.shrug, p.rollY * 0.28);
      arm.upper.rotation.set(ax, 0, az);
      arm.elbow.rotation.x = -el;
      arm.wrist.rotation.y = wr;
      arm.hand.scale.setScalar(p.hand);
    }
    const ph = t % 3.6;
    const blink = ph < 0.14 ? Math.sin((ph / 0.14) * Math.PI) : 0;
    for (const e of eyes) e.scale.y = Math.max(0.08, 1 - blink * 0.92);
  };

  const dispose = () => {
    scene.traverse((o) => {
      if (o instanceof THREE.Mesh) o.geometry.dispose();
    });
    for (const m of mats) m.dispose();
    shadowTex.dispose();
  };

  apply({ ...NEUTRAL }, 0);
  return { scene, apply, dispose };
}
