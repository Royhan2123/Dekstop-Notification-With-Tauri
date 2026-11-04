import { useCallback, useEffect, useState } from "react";
import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import { radarSummary } from "../viewmodel/radarSummary";

/** Token GitHub (hanya baca) untuk radar PR; token masuk brankas OS dan tidak bisa dibaca kembali. */
export function GithubSettings() {
  const t = useT();
  const [saved, setSaved] = useState(false);
  const [token, setToken] = useState("");
  const [status, setStatus] = useState("");

  const check = useCallback(async () => {
    setStatus("…");
    const r = await commands.githubRadar();
    if (r.status !== "ok") return setStatus(r.error);
    const { review, failing } = radarSummary(r.data);
    setStatus(
      `${review} ${t("settings.github.review")} · ${failing} ${t("settings.github.failing")}`,
    );
  }, [t]);

  useEffect(() => {
    void commands.githubTokenStatus().then((r) => {
      const has = r.status === "ok" && r.data;
      setSaved(has);
      if (has) void check();
    });
  }, [check]);

  async function save() {
    const r = await commands.githubTokenSet(token);
    if (r.status !== "ok") return setStatus(r.error);
    setToken("");
    setSaved(true);
    await check();
  }

  async function clear() {
    await commands.githubTokenClear();
    setSaved(false);
    setStatus("");
  }

  return (
    <fieldset className="field">
      <legend>{t("settings.github")}</legend>
      {saved ? (
        <div className="row">
          <span>{t("settings.github.saved")}</span>
          <button type="button" className="chip" onClick={() => void clear()}>
            {t("settings.github.remove")}
          </button>
        </div>
      ) : (
        <form
          className="row"
          onSubmit={(e) => {
            e.preventDefault();
            if (token.trim()) void save();
          }}
        >
          <input
            type="password"
            autoComplete="off"
            value={token}
            placeholder="github_pat_…"
            onChange={(e) => setToken(e.currentTarget.value)}
          />
          <button type="submit" className="chip" disabled={!token.trim()}>
            {t("settings.github.save")}
          </button>
        </form>
      )}
      {status ? <p>{status}</p> : <p className="hint">{t("settings.github.hint")}</p>}
    </fieldset>
  );
}
