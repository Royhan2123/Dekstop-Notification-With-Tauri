import { useRef } from "react";
import { useT } from "../../../shared/i18n";
import { readPhoto } from "../../../shared/theme/photo";
import { MASCOTS, THEMES, accentOf } from "../../../shared/theme/themes";
import { useSettingsVM } from "../../settings/viewmodel/useSettingsVM";

const FILE_VERSION = 1;
const MAX_FILE_BYTES = 600_000;

/** Pemilih tema: warna, maskot, foto latar, dan berkas tema yang bisa dibagikan. */
export function ThemeSettings() {
  const t = useT();
  const vm = useSettingsVM();
  const photoInput = useRef<HTMLInputElement>(null);
  const fileInput = useRef<HTMLInputElement>(null);
  if (!vm.settings) return null;
  const { theme, custom_accent, mascot, background } = vm.settings;
  const current = accentOf(theme, custom_accent);

  async function pickPhoto(file: File) {
    try {
      const { background, accent } = await readPhoto(file);
      await vm.update({ background, custom_accent: accent });
    } catch {
      vm.fail(t("settings.theme.photo_error"));
    }
  }

  function exportTheme() {
    const body = JSON.stringify({
      isla_theme: FILE_VERSION,
      theme,
      custom_accent,
      mascot,
      background,
    });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([body], { type: "application/json" }));
    a.download = "tema.isla-theme.json";
    a.click();
    URL.revokeObjectURL(a.href);
  }

  async function importTheme(file: File) {
    try {
      if (file.size > MAX_FILE_BYTES) throw new Error("size");
      const j: Record<string, unknown> = JSON.parse(await file.text());
      if (j.isla_theme !== FILE_VERSION) throw new Error("version");
      const str = (k: string) => (typeof j[k] === "string" ? (j[k] as string) : "");
      // Backend menyaring ulang semua nilai (warna, id, data URL gambar) sebelum disimpan.
      await vm.update({
        theme: str("theme") || "mint",
        custom_accent: str("custom_accent"),
        mascot: str("mascot") || "mino",
        background: str("background"),
      });
    } catch {
      vm.fail(t("settings.theme.file_error"));
    }
  }

  return (
    <fieldset className="field">
      <legend>{t("settings.theme")}</legend>
      <div className="swatches">
        {THEMES.map((th) => (
          <button
            key={th.id}
            type="button"
            className={!custom_accent && th.id === theme ? "swatch active" : "swatch"}
            style={{ background: th.accent }}
            title={t(`settings.theme.${th.id}`)}
            aria-label={t(`settings.theme.${th.id}`)}
            aria-pressed={!custom_accent && th.id === theme}
            onClick={() => void vm.update({ theme: th.id, custom_accent: "" })}
          />
        ))}
      </div>
      <label className="row">
        {t("settings.theme.custom")}
        <input
          type="color"
          value={current}
          onChange={(e) => void vm.update({ custom_accent: e.currentTarget.value })}
        />
      </label>
      <div className="row">
        {t("settings.mascot")}
        {MASCOTS.map((m) => (
          <button
            key={m}
            type="button"
            className={m === mascot ? "chip active" : "chip"}
            aria-pressed={m === mascot}
            onClick={() => void vm.update({ mascot: m })}
          >
            {t(`settings.mascot.${m}`)}
          </button>
        ))}
      </div>
      <div className="row">
        <button type="button" className="chip" onClick={() => photoInput.current?.click()}>
          {t("settings.theme.photo")}
        </button>
        {background && (
          <button type="button" className="chip" onClick={() => void vm.update({ background: "" })}>
            {t("settings.theme.photo_clear")}
          </button>
        )}
        <button type="button" className="chip" onClick={exportTheme}>
          {t("settings.theme.export")}
        </button>
        <button type="button" className="chip" onClick={() => fileInput.current?.click()}>
          {t("settings.theme.import")}
        </button>
      </div>
      <input
        ref={photoInput}
        type="file"
        accept="image/*"
        hidden
        onChange={(e) => {
          const f = e.currentTarget.files?.[0];
          e.currentTarget.value = "";
          if (f) void pickPhoto(f);
        }}
      />
      <input
        ref={fileInput}
        type="file"
        accept=".json,application/json"
        hidden
        onChange={(e) => {
          const f = e.currentTarget.files?.[0];
          e.currentTarget.value = "";
          if (f) void importTheme(f);
        }}
      />
    </fieldset>
  );
}
