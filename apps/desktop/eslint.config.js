import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist", "src/bindings.ts"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/features/**/view/**", "src/windows/**"],
    rules: {
      "no-restricted-imports": [
        "error",
        { paths: [{ name: "@tauri-apps/api/core", message: "View harus lewat ViewModel." }] },
      ],
    },
  },
);
