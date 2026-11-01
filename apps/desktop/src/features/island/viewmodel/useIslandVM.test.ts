import { expect, it } from "vitest";
import { shapeFor } from "./useIslandVM";
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

it("store menyimpan snapshot terbaru", () => {
  const next = { ...idleSnapshot, mode: "compact" as const };
  useIslandStore.getState().setSnapshot(next);
  expect(useIslandStore.getState().snapshot).toEqual(next);
});
