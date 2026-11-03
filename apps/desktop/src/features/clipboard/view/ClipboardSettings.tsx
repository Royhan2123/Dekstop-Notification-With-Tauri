import { useT } from "../../../shared/i18n";
import { useSettingsVM } from "../../settings/viewmodel/useSettingsVM";
import { useClipboardVM } from "../viewmodel/useClipboardVM";

/** Riwayat clipboard: opt-in, 20 salinan terakhir, dicari dan disalin kembali. */
export function ClipboardSettings() {
  const t = useT();
  const vm = useSettingsVM();
  const enabled = vm.settings?.clipboard_enabled ?? false;
  const clip = useClipboardVM(enabled);
  if (!vm.settings) return null;

  return (
    <fieldset className="field">
      <legend>{t("settings.clipboard")}</legend>
      <label className="row">
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => void vm.update({ clipboard_enabled: e.currentTarget.checked })}
        />
        {t("settings.clipboard.enabled")}
      </label>
      <p className="muted">{t("settings.clipboard.hint")}</p>
      {enabled && (
        <>
          <div className="row">
            <input
              value={clip.query}
              placeholder={t("settings.clipboard.search")}
              onChange={(e) => clip.setQuery(e.currentTarget.value)}
            />
            <button type="button" onClick={clip.clear}>
              {t("settings.clipboard.clear")}
            </button>
          </div>
          {clip.items.length === 0 ? (
            <p className="muted">{t("settings.clipboard.empty")}</p>
          ) : (
            <ul className="clip-list">
              {clip.items.map((item) => (
                <li key={item.id} className="row">
                  <code className="clip-text">{item.text}</code>
                  <button type="button" onClick={() => void clip.copy(item.id)}>
                    {t("settings.clipboard.copy")}
                  </button>
                </li>
              ))}
            </ul>
          )}
          {clip.error && (
            <p role="alert" className="error">
              {t("settings.error", clip.error)}
            </p>
          )}
        </>
      )}
    </fieldset>
  );
}
