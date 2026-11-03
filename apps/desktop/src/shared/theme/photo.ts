import { accentFromPixels } from "./themes";

const MAX_WIDTH = 480;
const MAX_BYTES = 350_000;

/** Warna aksen dari gambar yang sudah berupa data URL (sampul album, wallpaper). */
export async function accentOfImage(src: string): Promise<string> {
  const img = new Image();
  img.src = src;
  await img.decode();
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 32;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("canvas");
  ctx.drawImage(img, 0, 0, 32, 32);
  return accentFromPixels(ctx.getImageData(0, 0, 32, 32).data);
}

/** Foto pilihan pengguna -> latar (data URL JPEG kecil) dan warna aksen dari isinya. */
export async function readPhoto(file: File): Promise<{ background: string; accent: string }> {
  const bmp = await createImageBitmap(file);
  const scale = Math.min(1, MAX_WIDTH / bmp.width);
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(bmp.width * scale));
  canvas.height = Math.max(1, Math.round(bmp.height * scale));
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("canvas");
  ctx.drawImage(bmp, 0, 0, canvas.width, canvas.height);

  const tiny = document.createElement("canvas");
  tiny.width = tiny.height = 32;
  const tctx = tiny.getContext("2d");
  if (!tctx) throw new Error("canvas");
  tctx.drawImage(bmp, 0, 0, 32, 32);
  const accent = accentFromPixels(tctx.getImageData(0, 0, 32, 32).data);
  bmp.close();

  for (let q = 0.75; q >= 0.35; q -= 0.1) {
    const background = canvas.toDataURL("image/jpeg", q);
    if (background.length <= MAX_BYTES) return { background, accent };
  }
  throw new Error("too-big");
}
