import { expect, it } from "vitest";
import type { MediaDto } from "../../../shared/ipc";
import { formatTime, progressAt } from "./useMediaVM";

const base: MediaDto = {
  playing: true,
  position_ms: 10_000,
  duration_ms: 60_000,
  rate: 1,
  thumbnail: null,
  color: null,
  session_index: 0,
  session_count: 1,
};

it("progress bertambah saat diputar sesuai kecepatan dan berhenti di durasi", () => {
  expect(progressAt(base, 1000, 3000)).toBe(12_000);
  expect(progressAt({ ...base, rate: 2 }, 1000, 3000)).toBe(14_000);
  expect(progressAt(base, 0, 999_999)).toBe(60_000);
});

it("progress tidak bergerak saat dijeda", () => {
  expect(progressAt({ ...base, playing: false }, 1000, 9000)).toBe(10_000);
});

it("durasi tak diketahui tidak membatasi progress", () => {
  expect(progressAt({ ...base, duration_ms: 0 }, 0, 5000)).toBe(15_000);
});

it("format waktu menit:detik", () => {
  expect(formatTime(0)).toBe("0:00");
  expect(formatTime(65_000)).toBe("1:05");
});
