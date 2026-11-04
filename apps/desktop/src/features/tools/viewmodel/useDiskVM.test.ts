import { expect, it } from "vitest";
import { formatBytes, sharePercent } from "./useDiskVM";

it("ukuran berkas diringkas ke satuan yang pas", () => {
  expect(formatBytes(0)).toBe("0 B");
  expect(formatBytes(1023)).toBe("1023 B");
  expect(formatBytes(1536)).toBe("1.50 KB");
  expect(formatBytes(320 * 1024 ** 2)).toBe("320 MB");
  expect(formatBytes(12.5 * 1024 ** 3)).toBe("12.5 GB");
  expect(formatBytes(3 * 1024 ** 4)).toBe("3.00 TB");
});

it("lebar batang relatif terhadap total dan tetap di 0-100", () => {
  expect(sharePercent(50, 200)).toBe(25);
  expect(sharePercent(5, 0)).toBe(0);
  expect(sharePercent(300, 200)).toBe(100);
  expect(sharePercent(-1, 200)).toBe(0);
});
