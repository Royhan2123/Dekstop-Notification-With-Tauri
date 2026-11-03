import { describe, expect, it } from "vitest";
import { accentFromPixels, accentOf, paletteFor } from "./themes";

const px = (...rgb: number[]) =>
  new Uint8ClampedArray(rgb.flatMap((v, i) => (i % 3 === 2 ? [v, 255] : [v])));

describe("tema", () => {
  it("accentOf: warna sendiri menang, id tak dikenal jatuh ke mint", () => {
    expect(accentOf("ocean", "")).toBe("#38a3f5");
    expect(accentOf("ocean", "#ff00aa")).toBe("#ff00aa");
    expect(accentOf("tidak-ada", "")).toBe("#2fcf9c");
    expect(accentOf("ocean", "merah")).toBe("#38a3f5");
  });

  it("paletteFor menghasilkan semua variabel", () => {
    expect(Object.keys(paletteFor("#ff8a4c")).sort()).toEqual(
      [
        "--accent",
        "--accent-ink",
        "--m-eye",
        "--m-hi",
        "--m-lo",
        "--m-mid",
        "--m-soft",
        "--m-visor",
      ].sort(),
    );
  });

  it("accentFromPixels memilih warna dominan yang jenuh, bukan abu-abu", () => {
    // 1 piksel biru jenuh + banyak piksel abu-abu
    const gray = Array.from({ length: 50 }, () => [128, 128, 128, 255]).flat();
    const data = new Uint8ClampedArray([0, 60, 255, 255, ...gray]);
    const hex = accentFromPixels(data);
    const b = parseInt(hex.slice(5, 7), 16);
    const r = parseInt(hex.slice(1, 3), 16);
    expect(b).toBeGreaterThan(r + 40);
  });

  it("foto kosong atau hitam putih tetap menghasilkan hex valid", () => {
    expect(accentFromPixels(new Uint8ClampedArray())).toBe("#2fcf9c");
    expect(accentFromPixels(px(0, 0, 0))).toMatch(/^#[0-9a-f]{6}$/);
  });
});
