import { useState } from "react";
import { useT } from "../../../shared/i18n";
import { filterApps, useStartupVM } from "../viewmodel/useStartupVM";

/** Aplikasi yang dibuka otomatis saat Windows menyala: pilih dari aplikasi terpasang, atur urutan dan jeda. */
export function StartupTab() {
  const t = useT();
  const vm = useStartupVM();
  const [query, setQuery] = useState("");

  if (vm.installed) {
    const added = new Set(vm.apps.map((a) => a.path.toLowerCase()));
    const shown = filterApps(vm.installed, query);
    return (
      <div className="startup-tab">
        <div className="startup-pick-head">
          <button type="button" onClick={vm.closePicker}>
            ← {t("startup.back")}
          </button>
          <input
            type="search"
            autoFocus
            value={query}
            placeholder={t("startup.search")}
            aria-label={t("startup.search")}
            onChange={(e) => setQuery(e.currentTarget.value)}
          />
        </div>
        {vm.loading ? (
          <p className="shelf-empty">{t("startup.loading")}</p>
        ) : shown.length === 0 ? (
          <p className="shelf-empty">{t("startup.none")}</p>
        ) : (
          <ul className="startup-pick">
            {shown.map((a) => {
              const done = added.has(a.path.toLowerCase());
              return (
                <li key={a.path}>
                  <button
                    type="button"
                    disabled={done}
                    title={a.path}
                    onClick={() => void vm.add(a)}
                  >
                    <span className="startup-name">{a.name}</span>
                    <span className="startup-tick">{done ? `✓ ${t("startup.added")}` : "+"}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        )}
        {vm.error && <p className="shelf-empty">{vm.error}</p>}
      </div>
    );
  }

  return (
    <div className="startup-tab">
      <label className="startup-auto">
        <input
          type="checkbox"
          checked={vm.autostart}
          onChange={(e) => void vm.setAutostart(e.currentTarget.checked)}
        />
        <span>
          <b>{t("startup.autostart")}</b>
          <small>{t("startup.autostart_hint")}</small>
        </span>
      </label>

      {vm.apps.length === 0 ? (
        <p className="shelf-empty">{t("startup.empty")}</p>
      ) : (
        <ol className="startup-rows">
          {vm.apps.map((a, i) => (
            <li key={a.id}>
              <span className="startup-n">{i + 1}</span>
              <span className="startup-name" title={a.path}>
                {a.name}
              </span>
              <label className="startup-delay" title={t("startup.delay")}>
                <input
                  key={`${a.id}-${a.delay_secs}`}
                  type="number"
                  min={0}
                  max={300}
                  defaultValue={a.delay_secs}
                  aria-label={t("startup.delay")}
                  onBlur={(e) => {
                    const v = e.currentTarget.valueAsNumber;
                    if (v !== a.delay_secs) void vm.setDelay(a, v);
                  }}
                  onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
                />
                <small>{t("startup.sec")}</small>
              </label>
              <button
                type="button"
                className="row-act"
                aria-label={t("startup.up")}
                disabled={i === 0}
                onClick={() => void vm.move(a, true)}
              >
                ↑
              </button>
              <button
                type="button"
                className="row-act"
                aria-label={t("startup.down")}
                disabled={i === vm.apps.length - 1}
                onClick={() => void vm.move(a, false)}
              >
                ↓
              </button>
              <button
                type="button"
                className="row-act"
                aria-label={t("startup.remove")}
                onClick={() => void vm.remove(a)}
              >
                ×
              </button>
            </li>
          ))}
        </ol>
      )}

      {vm.error && <p className="shelf-empty">{vm.error}</p>}
      <div className="startup-foot">
        <button
          type="button"
          onClick={() => {
            setQuery("");
            void vm.openPicker();
          }}
        >
          + {t("startup.add")}
        </button>
        <button type="button" disabled={vm.apps.length === 0} onClick={() => void vm.runNow()}>
          ▶ {t("startup.run")}
        </button>
        {vm.started && <small>{t("startup.started")}</small>}
      </div>
    </div>
  );
}
