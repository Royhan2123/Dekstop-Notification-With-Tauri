import { describe, expect, it } from "vitest";
import {
  type MediaState,
  clampRate,
  mediaMessage,
  parseCommand,
  pickArtwork,
  pickTab,
  sanitizeState,
  searchUrl,
  tabOfSession,
} from "./protocol.js";

describe("protocol", () => {
  it("membatasi dan membulatkan kecepatan", () => {
    expect(clampRate(0.1)).toBe(0.25);
    expect(clampRate(9)).toBe(2);
    expect(clampRate(1.23)).toBe(1.25);
    expect(clampRate(NaN)).toBeNull();
  });

  it("mem-parse perintah sah dan menolak sisanya", () => {
    expect(parseCommand('{"type":"set_rate","rate":5}')).toEqual({ type: "set_rate", rate: 2 });
    expect(parseCommand('{"type":"search","query":"  lofi "}')).toEqual({
      type: "search",
      query: "lofi",
    });
    for (const bad of [
      "x",
      "null",
      '{"type":"search","query":" "}',
      '{"type":"set_rate","rate":"1"}',
      '{"type":"hapus"}',
    ]) {
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

describe("media dari browser", () => {
  it("perintah kontrol media diurai dan yang rusak ditolak", () => {
    expect(parseCommand('{"type":"media_control","id":"browser:12","action":"toggle"}')).toEqual({
      type: "media_control",
      tabId: 12,
      action: "toggle",
    });
    for (const bad of [
      '{"type":"media_control","id":"tab:12","action":"toggle"}',
      '{"type":"media_control","id":"browser:x","action":"toggle"}',
      '{"type":"media_control","id":"browser:1","action":"format_c"}',
      '{"type":"media_control","action":"next"}',
    ]) {
      expect(parseCommand(bad), bad).toBeNull();
    }
  });

  it("laporan content script dirapikan dan dibatasi; tanpa judul ditolak", () => {
    const s = sanitizeState({
      title: "  Lagu\nBaru ",
      artist: 5,
      playing: true,
      positionMs: -10,
      durationMs: 1e15,
      rate: 9,
      artwork: "https://x/a.jpg",
    });
    expect(s).toMatchObject({ title: "Lagu Baru", artist: "", positionMs: 0, rate: 2 });
    expect(s?.durationMs).toBe(86_400_000);
    expect(sanitizeState({ title: "   " })).toBeNull();
    expect(sanitizeState(null)).toBeNull();
  });

  it("pesan media membawa id sesi per tab dan dibatasi 8", () => {
    const states = new Map<number, MediaState>();
    for (let i = 1; i <= 12; i++) {
      states.set(i, {
        title: `t${i}`,
        artist: "a",
        playing: i === 1,
        positionMs: 5,
        durationMs: 9,
        rate: 1,
        artwork: "",
      });
    }
    const msg = mediaMessage(states);
    expect(msg.type).toBe("media");
    expect(msg.sessions).toHaveLength(8);
    expect(msg.sessions[0]).toMatchObject({
      id: "browser:1",
      title: "t1",
      playing: true,
      position_ms: 5,
    });
    expect(tabOfSession(msg.sessions[0].id)).toBe(1);
  });

  it("sampul terbesar dipilih", () => {
    expect(
      pickArtwork([
        { src: "kecil", sizes: "96x96" },
        { src: "besar", sizes: "512x512" },
        { src: "tanpa-ukuran" },
      ]),
    ).toBe("besar");
    expect(pickArtwork([])).toBe("");
  });
});
