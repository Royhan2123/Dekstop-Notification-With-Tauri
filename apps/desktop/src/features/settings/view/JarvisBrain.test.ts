import { expect, it } from "vitest";
import { installCommand } from "./JarvisBrain";

it("Windows memakai PowerShell, Mac dan Linux memakai bash, tanpa sudo", () => {
  expect(installCommand("Win32")).toContain("install.ps1 | iex");
  expect(installCommand("MacIntel")).toContain("install.sh | bash");
  expect(installCommand("Linux x86_64")).toContain("install.sh | bash");
  for (const p of ["Win32", "MacIntel", "Linux x86_64"]) {
    expect(installCommand(p)).not.toMatch(/sudo/);
  }
});
