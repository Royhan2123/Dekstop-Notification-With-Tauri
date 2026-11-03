import { useT } from "../../../shared/i18n";
import { THEMES, accentOf } from "../../../shared/theme/themes";
import { useSettingsVM } from "../../settings/viewmodel/useSettingsVM";

/** Pemilih tema: warna bawaan atau warna sendiri; pulau dan Mino mengikuti seketika. */
export function ThemeSettings() {
  const t = useT();
  const vm = useSettingsVM();
  if (!vm.settings) return null;
  const { theme, custom_accent } = vm.settings;
  const current = accentOf(theme, custom_accent);

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
    </fieldset>
  );
}
