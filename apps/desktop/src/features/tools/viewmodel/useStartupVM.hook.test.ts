// @vitest-environment jsdom
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";

const ok = { status: "ok", data: null } as const;
const apps = [
  { id: 1, name: "chrome", path: "C:\\chrome.exe", delay_secs: 0 },
  { id: 2, name: "code", path: "C:\\code.exe", delay_secs: 5 },
];
const cmds = vi.hoisted(() => ({
  startupList: vi.fn(),
  startupAdd: vi.fn(),
  startupInstalled: vi.fn(),
  startupRemove: vi.fn(),
  startupMove: vi.fn(),
  startupSetDelay: vi.fn(),
  startupRun: vi.fn(),
  settingsUpdate: vi.fn(),
}));
vi.mock("../../../shared/ipc", () => ({ commands: cmds }));

import { filterApps, useStartupVM } from "./useStartupVM";

beforeEach(() => {
  for (const f of Object.values(cmds)) f.mockReset();
  cmds.startupList.mockResolvedValue({ status: "ok", data: apps });
  for (const k of [
    "startupAdd",
    "startupRemove",
    "startupMove",
    "startupSetDelay",
    "startupRun",
  ] as const) {
    cmds[k].mockResolvedValue(ok);
  }
});

it("memuat daftar, lalu hapus/geser/jeda dikirim ke backend dan daftar dimuat ulang", async () => {
  const { result } = renderHook(() => useStartupVM());
  await waitFor(() => expect(result.current.apps).toHaveLength(2));
  await act(() => result.current.move(apps[1], true));
  expect(cmds.startupMove).toHaveBeenCalledWith(2, true);
  await act(() => result.current.remove(apps[0]));
  expect(cmds.startupRemove).toHaveBeenCalledWith(1);
  expect(cmds.startupList.mock.calls.length).toBeGreaterThanOrEqual(3);
});

it("jeda negatif atau bukan angka jadi 0 dan pecahan dibulatkan", async () => {
  const { result } = renderHook(() => useStartupVM());
  await waitFor(() => expect(result.current.apps).toHaveLength(2));
  await act(() => result.current.setDelay(apps[0], -4));
  expect(cmds.startupSetDelay).toHaveBeenLastCalledWith(1, 0);
  await act(() => result.current.setDelay(apps[0], Number.NaN));
  expect(cmds.startupSetDelay).toHaveBeenLastCalledWith(1, 0);
  await act(() => result.current.setDelay(apps[0], 7.6));
  expect(cmds.startupSetDelay).toHaveBeenLastCalledWith(1, 8);
});

it("galat dari backend ditampilkan; coba jalankan menandai sudah dimulai hanya bila berhasil", async () => {
  cmds.startupRun.mockResolvedValueOnce({ status: "error", error: "sedang dijalankan" });
  const { result } = renderHook(() => useStartupVM());
  await waitFor(() => expect(result.current.apps).toHaveLength(2));
  await act(() => result.current.runNow());
  expect(result.current.error).toBe("sedang dijalankan");
  expect(result.current.started).toBe(false);
  await act(() => result.current.runNow());
  expect(result.current.error).toBeNull();
  expect(result.current.started).toBe(true);
});

it("pemilih memuat aplikasi terpasang, menambahkan pilihan, dan bisa ditutup", async () => {
  const chrome = { name: "Google Chrome", path: "C:\\chrome.exe" };
  cmds.startupInstalled.mockResolvedValue({ status: "ok", data: [chrome] });
  const { result } = renderHook(() => useStartupVM());
  await waitFor(() => expect(result.current.apps).toHaveLength(2));
  expect(result.current.installed).toBeNull();
  await act(() => result.current.openPicker());
  expect(result.current.installed).toEqual([chrome]);
  await act(() => result.current.add(chrome));
  expect(cmds.startupAdd).toHaveBeenCalledWith("Google Chrome", "C:\\chrome.exe");
  act(() => result.current.closePicker());
  expect(result.current.installed).toBeNull();
});

it("pencarian aplikasi mengabaikan huruf besar/kecil dan spasi di ujung", () => {
  const list = [
    { name: "Google Chrome", path: "a" },
    { name: "Visual Studio Code", path: "b" },
  ];
  expect(filterApps(list, "  CHROME ")).toEqual([list[0]]);
  expect(filterApps(list, "")).toEqual(list);
  expect(filterApps(list, "zzz")).toEqual([]);
});
