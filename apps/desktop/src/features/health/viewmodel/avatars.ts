import type { MessageKey } from "../../../shared/i18n";

/** Satu karakter 3D: warna dan ciri khasnya. Bentuk dasarnya sama (kerangka di avatarRig). */
export interface AvatarSpec {
  id: "robo" | "mimi" | "beno" | "astro" | "alien";
  name: MessageKey;
  skin: number; // kepala
  body: number; // badan dan lengan
  accent: number; // pipi/antena/ransel
  eye: number;
  ears: "none" | "cat" | "bear";
  extra: "none" | "antenna" | "helmet" | "antennae";
  faceplate: boolean; // layar gelap di wajah (robot)
}

export const AVATARS: readonly AvatarSpec[] = [
  {
    id: "robo",
    name: "health.stretch.avatar.robo",
    skin: 0xdfe8ff,
    body: 0x6c8cff,
    accent: 0xffd166,
    eye: 0x7df9ff,
    ears: "none",
    extra: "antenna",
    faceplate: true,
  },
  {
    id: "mimi",
    name: "health.stretch.avatar.mimi",
    skin: 0xffb26b,
    body: 0xff8f5a,
    accent: 0xff8fb1,
    eye: 0x2b2118,
    ears: "cat",
    extra: "none",
    faceplate: false,
  },
  {
    id: "beno",
    name: "health.stretch.avatar.beno",
    skin: 0xb98562,
    body: 0x8f6244,
    accent: 0xe8c9a8,
    eye: 0x20140c,
    ears: "bear",
    extra: "none",
    faceplate: false,
  },
  {
    id: "astro",
    name: "health.stretch.avatar.astro",
    skin: 0xffd9bd,
    body: 0xf2f4ff,
    accent: 0xff8a3d,
    eye: 0x232a3a,
    ears: "none",
    extra: "helmet",
    faceplate: false,
  },
  {
    id: "alien",
    name: "health.stretch.avatar.alien",
    skin: 0x7be0a0,
    body: 0x4fbf82,
    accent: 0xc77dff,
    eye: 0x16251c,
    ears: "none",
    extra: "antennae",
    faceplate: false,
  },
];

const KEY = "isla.stretch.avatar";

export function savedAvatar(): AvatarSpec["id"] {
  try {
    const id = localStorage.getItem(KEY);
    return AVATARS.find((a) => a.id === id)?.id ?? AVATARS[0].id;
  } catch {
    return AVATARS[0].id;
  }
}

export function saveAvatar(id: AvatarSpec["id"]) {
  try {
    localStorage.setItem(KEY, id);
  } catch {
    // Penyimpanan penuh atau dimatikan: pilihan hanya berlaku sesi ini.
  }
}
