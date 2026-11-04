import { useEffect, useState } from "react";
import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { useSettingsVM } from "../../settings/viewmodel/useSettingsVM";

/** Tautan kalender iCal untuk pengingat rapat lima menit sebelum mulai, dengan status rapat berikutnya. */
export function CalendarSettings() {
  const t = useT();
  const vm = useSettingsVM();
  const saved = vm.settings?.calendar_url ?? "";
  const [url, setUrl] = useState(saved);
  const [status, setStatus] = useState("");

  useEffect(() => setUrl(saved), [saved]);

  useEffect(() => {
    if (!saved) return setStatus("");
    let alive = true;
    setStatus("…");
    void commands.calendarNext().then((r) => {
      if (!alive) return;
      if (r.status !== "ok") setStatus(r.error);
      else if (!r.data) setStatus(t("settings.calendar.none"));
      else
        setStatus(`${t("settings.calendar.next")} ${r.data.title} · ${r.data.date} ${r.data.time}`);
    });
    return () => {
      alive = false;
    };
  }, [saved, t]);

  const save = () => {
    const next = url.trim();
    if (next !== saved) void vm.update({ calendar_url: next });
  };

  return (
    <fieldset className="field">
      <legend>{t("settings.calendar")}</legend>
      <input
        type="url"
        value={url}
        placeholder="https://calendar.google.com/calendar/ical/…/basic.ics"
        onChange={(e) => setUrl(e.currentTarget.value)}
        onBlur={save}
        onKeyDown={(e) => e.key === "Enter" && save()}
      />
      {status ? <p>{status}</p> : <p className="hint">{t("settings.calendar.hint")}</p>}
    </fieldset>
  );
}
