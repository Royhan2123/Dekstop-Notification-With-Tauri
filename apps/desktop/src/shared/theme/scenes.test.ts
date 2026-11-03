import { describe, expect, it } from "vitest";
import { imageScene, timeScene } from "./scenes";

describe("suasana latar", () => {
  it("jam menentukan aksen: fajar, siang, senja, malam", () => {
    expect(timeScene(6).accent).toBe("#ff9a6c");
    expect(timeScene(12).accent).toBe("#38a3f5");
    expect(timeScene(18).accent).toBe("#ff8a4c");
    expect(timeScene(23).accent).toBe("#8aa4ff");
    expect(timeScene(2).accent).toBe("#8aa4ff");
  });

  it("batas jam jatuh ke suasana berikutnya", () => {
    expect(timeScene(4).accent).toBe(timeScene(23).accent);
    expect(timeScene(5).accent).toBe(timeScene(6).accent);
    expect(timeScene(8).accent).toBe(timeScene(12).accent);
    expect(timeScene(17).accent).toBe(timeScene(18).accent);
    expect(timeScene(20).accent).toBe(timeScene(23).accent);
  });

  it("latar gambar diberi lapisan gelap", () => {
    const s = imageScene("data:image/jpeg;base64,AAA", "#123456");
    expect(s.background).toContain('url("data:image/jpeg;base64,AAA")');
    expect(s.background).toContain("rgba(0,0,0,.62)");
    expect(s.accent).toBe("#123456");
  });
});
