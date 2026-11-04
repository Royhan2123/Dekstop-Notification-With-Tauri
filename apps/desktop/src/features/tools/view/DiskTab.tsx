import type { CSSProperties } from "react";
import { useT } from "../../../shared/i18n";
import { formatBytes, sharePercent, useDiskVM } from "../viewmodel/useDiskVM";

/** Ukuran folder/berkas di disk. Hanya melihat dan membuka di Explorer; tidak ada tombol hapus. */
export function DiskTab() {
  const t = useT();
  const vm = useDiskVM();
  const s = vm.status;
  const scanning = s?.running === true;

  return (
    <div className="disk-tab">
      <div className="disk-nav">
        {vm.drives.map((d) => (
          <button
            key={d}
            type="button"
            className={s?.path.startsWith(d) ? "on" : ""}
            onClick={() => void vm.scan(d)}
          >
            {d.replace(/\\$/, "")}
          </button>
        ))}
        {s?.parent && (
          <button
            type="button"
            aria-label={t("disk.up")}
            title={t("disk.up")}
            onClick={() => void vm.scan(s.parent ?? "")}
          >
            ↑
          </button>
        )}
        {s?.path && <span className="disk-path">{s.path}</span>}
      </div>

      {!s?.path && <p className="shelf-empty">{t("disk.pick")}</p>}

      {scanning && (
        <div className="disk-progress" role="status">
          <span className="disk-bar indet" />
          <small>{t("disk.scanning", s.files_scanned.toLocaleString())}</small>
        </div>
      )}

      {s?.done && !scanning && (
        <>
          <p className="tools-sec">{t("disk.total", formatBytes(s.total))}</p>
          {s.entries.length === 0 ? (
            <p className="shelf-empty">{t("disk.empty")}</p>
          ) : (
            <ul className="disk-rows">
              {s.entries.map((e, i) => (
                <li key={e.path} style={{ "--i": Math.min(i, 12) } as CSSProperties}>
                  <button
                    type="button"
                    className="disk-name"
                    disabled={!e.is_dir}
                    title={e.path}
                    onClick={() => void vm.scan(e.path)}
                  >
                    <span aria-hidden="true">{e.is_dir ? "📁" : "📄"}</span>
                    <span className="disk-label">{e.name}</span>
                    <span
                      className="disk-bar"
                      style={{ "--w": `${sharePercent(e.size, s.total)}%` } as CSSProperties}
                    />
                  </button>
                  <span className="disk-size">{formatBytes(e.size)}</span>
                  <button
                    type="button"
                    className="row-act"
                    aria-label={t("disk.reveal")}
                    title={t("disk.reveal")}
                    onClick={() => void vm.reveal(e.path)}
                  >
                    ↗
                  </button>
                </li>
              ))}
            </ul>
          )}
        </>
      )}

      {vm.error && <p className="shelf-empty">{vm.error}</p>}
      <small className="disk-hint">{t("disk.hint")}</small>
    </div>
  );
}
