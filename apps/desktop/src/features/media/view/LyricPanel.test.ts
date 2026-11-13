import { expect, it } from "vitest";
import { SHIFT_STEP_MS, formatOffset } from "./LyricPanel";

it("selisih lirik ditulis ringkas dengan tanda dan koma desimal", () => {
  expect(formatOffset(0)).toBe("0");
  expect(formatOffset(250)).toBe("+0,25s");
  expect(formatOffset(500)).toBe("+0,5s");
  expect(formatOffset(-1000)).toBe("-1s");
  expect(formatOffset(-1250)).toBe("-1,25s");
  expect(formatOffset(5000)).toBe("+5s");
});

it("langkah geser berlipat agar kembali ke nol bisa dicapai", () => {
  expect(4 * SHIFT_STEP_MS).toBe(1000);
  expect(formatOffset(4 * SHIFT_STEP_MS - 4 * SHIFT_STEP_MS)).toBe("0");
});
