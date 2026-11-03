import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { useSettingsStore } from "../../settings/store";
import { useClipboardVM } from "../viewmodel/useClipboardVM";

/** Riwayat clipboard di pulau: ketuk satu baris untuk menyalin kembali. */
export function ClipboardPanel({ onClose }: { onClose: () => void }) {
  const t = useT();
  const enabled = useSettingsStore((s) => s.settings?.clipboard_enabled === true);
  const clip = useClipboardVM(enabled);

  return (
    <div className="clip-panel" onClick={(e) => e.stopPropagation()}>
      <div className="shelf-head">
        <span>{t("dock.clipboard")}</span>
        {enabled && (
          <button type="button" onClick={clip.clear} disabled={!clip.items.length}>
            {t("settings.clipboard.clear")}
          </button>
        )}
        <button type="button" aria-label={t("island.dismiss")} onClick={onClose}>
          ×
        </button>
      </div>
      {!enabled ? (
        <p className="shelf-empty">
          {t("dock.clipboard_off")}{" "}
          <button type="button" onClick={() => void commands.openSettings()}>
            {t("dock.settings")}
          </button>
        </p>
      ) : clip.items.length === 0 ? (
        <p className="shelf-empty">{t("settings.clipboard.empty")}</p>
      ) : (
        <ul className="clip-rows">
          {clip.items.map((item) => (
            <li key={item.id}>
              <button type="button" title={item.text} onClick={() => void clip.copy(item.id)}>
                {item.text}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
