import { expect, it } from "vitest";
import { dictionaries, translate } from "./index";

it("id dan en punya kunci sama dan tidak ada teks kosong", () => {
  expect(Object.keys(dictionaries.en).sort()).toEqual(Object.keys(dictionaries.id).sort());
  for (const dict of Object.values(dictionaries)) {
    for (const text of Object.values(dict)) expect(text.trim()).not.toBe("");
  }
});

it("translate mengganti placeholder berurutan", () => {
  expect(translate("id", "onboarding.step", [2, 4])).toBe("Langkah 2 dari 4");
  expect(translate("en", "onboarding.step", [2, 4])).toBe("Step 2 of 4");
});
