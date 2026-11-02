import { describe, expect, it } from "vitest";
import { clampRate, parseCommand, pickTab, searchUrl } from "./protocol.js";

describe("protocol", () => {
  it("membatasi dan membulatkan kecepatan", () => {
    expect(clampRate(0.1)).toBe(0.25);
    expect(clampRate(9)).toBe(2);
    expect(clampRate(1.23)).toBe(1.25);
    expect(clampRate(NaN)).toBeNull();
  });

  it("mem-parse perintah sah dan menolak sisanya", () => {
    expect(parseCommand('{"type":"set_rate","rate":5}')).toEqual({ type: "set_rate", rate: 2 });
    expect(parseCommand('{"type":"search","query":"  lofi "}')).toEqual({ type: "search", query: "lofi" });
    for (const bad of ["x", "null", '{"type":"search","query":" "}', '{"type":"set_rate","rate":"1"}', '{"type":"hapus"}']) {
      expect(parseCommand(bad)).toBeNull();
    }
  });

  it("membuat URL pencarian ter-encode", () => {
    expect(searchUrl("a&b c")).toBe("https://www.youtube.com/results?search_query=a%26b%20c");
  });

  it("memilih tab yang bersuara", () => {
    expect(pickTab([{ audible: false }, { audible: true }])).toEqual({ audible: true });
    expect(pickTab([{ audible: false }])).toEqual({ audible: false });
    expect(pickTab([])).toBeUndefined();
  });
});
