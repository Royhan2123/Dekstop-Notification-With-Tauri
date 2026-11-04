import { expect, it } from "vitest";
import { COUNTDOWN_SECS, powerReducer, type PowerStep } from "./usePowerVM";

const idle: PowerStep = { phase: "idle" };

it("matikan harus lewat konfirmasi lalu hitung mundur, tidak langsung jalan", () => {
  const confirm = powerReducer(idle, { type: "pick", action: "shutdown" });
  expect(confirm).toEqual({ phase: "confirm", action: "shutdown" });
  const counting = powerReducer(confirm, { type: "confirm" });
  expect(counting).toEqual({ phase: "counting", action: "shutdown", left: COUNTDOWN_SECS });
});

it("konfirmasi tidak bisa dilompati dan tick tidak menghidupkan hitung mundur dari idle", () => {
  expect(powerReducer(idle, { type: "confirm" })).toEqual(idle);
  expect(powerReducer(idle, { type: "tick" })).toEqual(idle);
  const confirm = powerReducer(idle, { type: "pick", action: "restart" });
  expect(powerReducer(confirm, { type: "tick" })).toEqual(confirm);
});

it("hitung mundur berhenti di nol dan batal kapan saja kembali ke idle", () => {
  let s: PowerStep = { phase: "counting", action: "sleep", left: 1 };
  s = powerReducer(s, { type: "tick" });
  expect(s).toEqual({ phase: "counting", action: "sleep", left: 0 });
  expect(powerReducer(s, { type: "tick" })).toEqual(s);
  expect(powerReducer(s, { type: "cancel" })).toEqual(idle);
  const mid: PowerStep = { phase: "counting", action: "restart", left: 3 };
  expect(powerReducer(mid, { type: "cancel" })).toEqual(idle);
});

it("kunci layar tidak masuk mesin konfirmasi", () => {
  expect(powerReducer(idle, { type: "pick", action: "lock" })).toEqual(idle);
});
