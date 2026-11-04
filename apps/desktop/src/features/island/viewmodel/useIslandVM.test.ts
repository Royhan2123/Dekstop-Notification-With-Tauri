import { expect, it } from "vitest";
import { EXPANDED_W, shapeFor } from "./useIslandVM";
import { idleSnapshot, useIslandStore } from "../store";

it("tiap mode punya ukuran yang makin besar, hidden tidak terlihat", () => {
  const hidden = shapeFor({ ...idleSnapshot, mode: "hidden" });
  const idle = shapeFor(idleSnapshot);
  const compact = shapeFor({ ...idleSnapshot, mode: "compact" });
  const expanded = shapeFor({ ...idleSnapshot, mode: "expanded" });
  expect(hidden.opacity).toBe(0);
  expect(idle.width).toBeLessThan(compact.width);
  expect(compact.width).toBeLessThan(expanded.width);
});

it("hover melebarkan pulau idle sedikit", () => {
  expect(shapeFor({ ...idleSnapshot, hovered: true }).width).toBeGreaterThan(
    shapeFor(idleSnapshot).width,
  );
});

it("semua kartu terbuka selebar sama (musik, Tanya AI, rak, polos)", () => {
  const open = { ...idleSnapshot, mode: "expanded" as const };
  const withKind = (kind: "media" | "ask" | "shelf") => ({
    ...open,
    content: { kind, text: "t", detail: "a" },
  });
  for (const s of [open, withKind("media"), withKind("ask"), withKind("shelf")]) {
    expect(shapeFor(s).width).toBe(EXPANDED_W);
  }
});

it("kartu Tanya AI memanjang bila ada riwayat chat", () => {
  const ask = {
    ...idleSnapshot,
    mode: "expanded" as const,
    content: { kind: "ask" as const, text: "", detail: "input" },
  };
  expect(shapeFor(ask, { chat: true }).height).toBeGreaterThan(shapeFor(ask).height);
  expect(shapeFor({ ...ask, content: { ...ask.content, detail: "done" } }).height).toBe(
    shapeFor(ask, { chat: true }).height,
  );
});

it("store menyimpan snapshot terbaru", () => {
  const next = { ...idleSnapshot, mode: "compact" as const };
  useIslandStore.getState().setSnapshot(next);
  expect(useIslandStore.getState().snapshot).toEqual(next);
});
