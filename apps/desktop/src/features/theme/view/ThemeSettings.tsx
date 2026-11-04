import { useRef, useState } from "react";
import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { readPhoto } from "../../../shared/theme/photo";
import { PRESETS, renderPreset } from "../../../shared/theme/presets";
import { MASCOTS, THEMES, accentOf } from "../../../shared/theme/themes";
import { useSettingsVM } from "../../settings/viewmodel/useSettingsVM";

const FILE_VERSION = 1;
const BACKGROUND_MODES = ["static", "album", "time", "wallpaper"] as const;
const MAX_FILE_BYTES = 600_000;

/** Pemilih tema: warna, maskot, foto latar, dan berkas tema yang bisa dibagikan. */
export function ThemeSettings() {
  const t = useT();
  const vm = useSettingsVM();
  const photoInput = useRef<HTMLInputElement>(null);
  const fileInput = useRef<HTMLInputElement>(null);
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);
  if (!vm.settings) return null;
  const { theme, custom_accent, mascot, background, background_mode } = vm.settings;
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
    if (file.size > MAX_FILE_BYTES) return vm.fail(t("settings.theme.file_error"));
    await applyThemeText(await file.text());
  }

  async function importFromUrl() {
    setBusy(true);
    try {
      const r = await commands.themeFetch(url);
      if (r.status !== "ok") return vm.fail(r.error);
      await applyThemeText(r.data);
      setUrl("");
    } finally {
      setBusy(false);
    }
  }

  async function applyThemeText(text: string) {
    try {
      const j: Record<string, unknown> = JSON.parse(text);
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
      <div className="theme-preview" aria-hidden="true">
        <span className="pv-cover" />
        <div>
          <b>Isla Desk</b>
          <small>{t("settings.theme.preview")}</small>
          <div className="pv-bar" />
        </div>
      </div>
      <div className="presets">
        {PRESETS.map((p) => {
          const src = renderPreset(p);
          const active = background === src;
          return (
            <button
              key={p.id}
              type="button"
              className={active ? "preset active" : "preset"}
              style={{ backgroundImage: `url(${src})` }}
              aria-pressed={active}
              onClick={() =>
                void vm.update({
                  background: src,
                  custom_accent: p.accent,
                  background_mode: "static",
                })
              }
            >
              <span>{p.name}</span>
            </button>
          );
        })}
      </div>
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
      <div className="column">
        {t("settings.theme.mode")}
        <div className="mode-grid">
          {BACKGROUND_MODES.map((m) => (
            <button
              key={m}
              type="button"
              className={m === background_mode ? "chip active" : "chip"}
              aria-pressed={m === background_mode}
              onClick={() => void vm.update({ background_mode: m })}
            >
              {t(`settings.theme.mode.${m}`)}
            </button>
          ))}
        </div>
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
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          if (url.trim() && !busy) void importFromUrl();
        }}
      >
        <input
          type="url"
          value={url}
          placeholder={t("settings.theme.url_placeholder")}
          onChange={(e) => setUrl(e.currentTarget.value)}
        />
        <button type="submit" className="chip" disabled={busy || !url.trim()}>
          {t("settings.theme.url_import")}
        </button>
      </form>
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
