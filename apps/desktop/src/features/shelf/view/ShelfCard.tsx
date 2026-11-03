import { useState } from "react";
import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { useIslandStore } from "../../island/store";

/** Rak file: berkas yang dijatuhkan ke pulau, bisa dibuka, ditunjukkan, disalin (Ctrl+V), atau dibuang. */
export function ShelfCard() {
  const t = useT();
  const items = useIslandStore((s) => s.snapshot.shelf);
  const [copied, setCopied] = useState<string | null>(null);

  const copy = async (path: string) => {
    if (await commands.shelfCopy(path)) {
      setCopied(path);
      window.setTimeout(() => setCopied(null), 1500);
    }
  };

  return (
    <div className="shelf-card" onClick={(e) => e.stopPropagation()}>
      <div className="shelf-head">
        <span>{t("island.shelf")}</span>
        <button type="button" onClick={() => void commands.shelfClear()} disabled={!items.length}>
          {t("shelf.clear")}
        </button>
        <button
          type="button"
          aria-label={t("island.dismiss")}
          onClick={() => void commands.islandDismiss()}
        >
          ×
        </button>
      </div>
      {items.length === 0 ? (
        <p className="shelf-empty">{t("shelf.empty")}</p>
      ) : (
        <ul className="shelf-list">
          {items.map((it) => (
            <li key={it.path} title={it.path}>
              <span className="shelf-name">{it.name}</span>
              <button type="button" onClick={() => void commands.shelfOpenFile(it.path)}>
                {t("shelf.open")}
              </button>
              <button type="button" onClick={() => void commands.shelfReveal(it.path)}>
                {t("shelf.reveal")}
              </button>
              <button type="button" onClick={() => void copy(it.path)}>
                {copied === it.path ? t("shelf.copied") : t("shelf.copy")}
              </button>
              <button
                type="button"
                aria-label={t("shelf.remove")}
                onClick={() => void commands.shelfRemove(it.path)}
              >
                ×
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
