import { expect, it } from "vitest";
import { PRESETS } from "./presets";

it("preset punya id unik dan warna hex enam digit (aksen harus lolos saringan backend)", () => {
  const hex = /^#[0-9a-f]{6}$/i;
  expect(new Set(PRESETS.map((p) => p.id)).size).toBe(PRESETS.length);
  for (const p of PRESETS) {
    expect(p.accent).toMatch(hex);
    expect(p.base).toMatch(hex);
    for (const b of p.blobs) expect(b.color).toMatch(hex);
  }
});
