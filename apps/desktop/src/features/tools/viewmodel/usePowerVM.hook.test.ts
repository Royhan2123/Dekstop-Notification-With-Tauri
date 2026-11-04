// @vitest-environment jsdom
import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

const powerPerform = vi.hoisted(() =>
  vi.fn<(action: string) => Promise<{ status: "ok"; data: null }>>(),
);
vi.mock("../../../shared/ipc", () => ({ commands: { powerPerform } }));

import { COUNTDOWN_SECS, usePowerVM } from "./usePowerVM";

beforeEach(() => {
  vi.useFakeTimers();
  powerPerform.mockReset();
  powerPerform.mockResolvedValue({ status: "ok", data: null });
});
afterEach(() => vi.useRealTimers());

// Efek React menjadwalkan timer berikutnya hanya setelah render, jadi maju satu detik per putaran act.
const tick = async (secs: number) => {
  for (let i = 0; i < secs; i++) await act(() => vi.advanceTimersByTimeAsync(1000));
};

it("matikan: setelah dikonfirmasi dan hitung mundur habis, perintah daya dikirim sekali", async () => {
  const { result } = renderHook(() => usePowerVM());
  act(() => result.current.pick("shutdown"));
  await tick(10);
  expect(powerPerform).not.toHaveBeenCalled(); // masih menunggu persetujuan
  act(() => result.current.confirm());
  await tick(COUNTDOWN_SECS - 1);
  expect(powerPerform).not.toHaveBeenCalled();
  await tick(1);
  expect(powerPerform).toHaveBeenCalledTimes(1);
  expect(powerPerform).toHaveBeenCalledWith("shutdown");
  expect(result.current.step).toEqual({ phase: "idle" });
});

it("batal saat hitung mundur tidak mengirim apa pun", async () => {
  const { result } = renderHook(() => usePowerVM());
  act(() => result.current.pick("restart"));
  act(() => result.current.confirm());
  await tick(2);
  act(() => result.current.cancel());
  await tick(20);
  expect(powerPerform).not.toHaveBeenCalled();
});

it("tidur dan mulai ulang dikirim dengan aksinya; kunci langsung tanpa konfirmasi", async () => {
  const { result } = renderHook(() => usePowerVM());
  act(() => result.current.pick("sleep"));
  act(() => result.current.confirm());
  await tick(COUNTDOWN_SECS);
  expect(powerPerform).toHaveBeenLastCalledWith("sleep");
  act(() => result.current.pick("lock"));
  await act(() => Promise.resolve());
  expect(powerPerform).toHaveBeenLastCalledWith("lock");
  expect(result.current.step).toEqual({ phase: "idle" });
});
