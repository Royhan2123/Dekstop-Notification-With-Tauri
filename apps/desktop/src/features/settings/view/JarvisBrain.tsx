import { useEffect, useState } from "react";
import { useT } from "../../../shared/i18n";
import { commands } from "../../../shared/ipc";
import type { JarvisAi } from "../../../bindings";

const BASE = "https://raw.githubusercontent.com/Royhan2123/jarvis/main/scripts";
/** Perintah pemasangan satu baris sesuai OS (Windows: PowerShell; lainnya: bash). */
export function installCommand(platform = navigator.platform): string {
  return /win/i.test(platform) && !/darwin/i.test(platform)
    ? `irm ${BASE}/install.ps1 | iex`
    : `curl -LsSf ${BASE}/install.sh | bash`;
}

const ENGINE = {
  fish: "settings.jarvis.engine.fish",
  piper: "settings.jarvis.engine.piper",
  system: "settings.jarvis.engine.system",
  edge: "settings.jarvis.engine.edge",
} as const;
const GENDER = {
  male: "settings.jarvis.gender.male",
  female: "settings.jarvis.gender.female",
} as const;

type Check = { state: "idle" | "busy" | "ok" | "error"; text: string };

/** Setelan otak Jarvis (Ollama bawaan atau API online). Kunci hanya ditulis ke Jarvis, tidak dibaca kembali. */
export function JarvisBrain() {
  const t = useT();
  const [info, setInfo] = useState<JarvisAi | null>(null);
  const [provider, setProvider] = useState("ollama");
  const [model, setModel] = useState("");
  const [base, setBase] = useState("");
  const [key, setKey] = useState("");
  const [installed, setInstalled] = useState<boolean | null>(null);
  const [started, setStarted] = useState<"idle" | "busy" | "copied">("idle");
  const [check, setCheck] = useState<Check>({ state: "idle", text: "" });

  function load(r: Awaited<ReturnType<typeof commands.jarvisAiGet>>) {
    if (r.status === "ok") {
      setInfo(r.data);
      setProvider(r.data.provider);
      setModel(r.data.provider === "ollama" ? "" : r.data.fast_model);
      setBase(r.data.api_base);
      setCheck({ state: "idle", text: "" });
    } else {
      setInfo(null);
      setCheck({ state: "error", text: t("settings.jarvis.offline") });
    }
  }

  useEffect(() => {
    void commands.jarvisAiGet().then(load);
    void commands.jarvisInstalled().then(setInstalled);
  }, []);

  const online = provider !== "ollama";

  async function start() {
    setStarted("busy");
    const r = await commands.jarvisStart();
    if (r.status !== "ok") {
      setStarted("idle");
      return setCheck({ state: "error", text: r.error });
    }
    // Jarvis butuh beberapa detik sampai API-nya siap: coba lagi sampai terhubung.
    for (let i = 0; i < 30; i++) {
      const g = await commands.jarvisAiGet();
      if (g.status === "ok") {
        setStarted("idle");
        return load(g);
      }
      await new Promise((ok) => setTimeout(ok, 2000));
    }
    setStarted("idle");
    setCheck({ state: "error", text: t("settings.jarvis.offline") });
  }

  async function copy() {
    await navigator.clipboard.writeText(installCommand());
    setStarted("copied");
  }

  async function apply() {
    setCheck({ state: "busy", text: "" });
    const r = await commands.jarvisAiSet(
      provider,
      key.trim() || null,
      online ? base.trim() || null : null,
      online ? model.trim() || null : null,
    );
    if (r.status !== "ok") return setCheck({ state: "error", text: r.error });
    setKey("");
    setInfo(r.data);
    const test = await commands.jarvisAiTest();
    setCheck(
      test.status === "ok"
        ? {
            state: "ok",
            text: `${t("settings.jarvis.applied")}${test.data ? `: ${test.data}` : ""}`,
          }
        : { state: "error", text: test.error },
    );
  }

  return (
    <fieldset className="field">
      <legend>{t("settings.jarvis")}</legend>
      <p className="hint">{t("settings.jarvis.hint")}</p>
      <p role="status">
        {info === null ? "○ " : "● "}
        <strong>{info === null ? t("settings.jarvis.off") : t("settings.jarvis.on")}</strong>
        {info !== null && info.voice_engine && (
          <>
            {" · "}
            {t("settings.jarvis.voice")}:{" "}
            {info.voice_engine in ENGINE
              ? t(ENGINE[info.voice_engine as keyof typeof ENGINE])
              : info.voice_engine}
            {info.voice_gender &&
              `, ${info.voice_gender in GENDER ? t(GENDER[info.voice_gender as keyof typeof GENDER]) : info.voice_gender}`}
          </>
        )}
      </p>
      {info === null ? (
        <>
          {check.state === "error" && <span role="status">✗ {check.text}</span>}
          {installed === false ? (
            <div className="column">
              <p className="hint">{t("settings.jarvis.not_installed")}</p>
              <code>{installCommand()}</code>
              <button type="button" onClick={() => void copy()}>
                {started === "copied" ? t("settings.jarvis.copied") : t("settings.jarvis.copy")}
              </button>
            </div>
          ) : (
            installed && (
              <div className="row">
                <button type="button" disabled={started === "busy"} onClick={() => void start()}>
                  {started === "busy" ? t("settings.jarvis.starting") : t("settings.jarvis.start")}
                </button>
              </div>
            )
          )}
        </>
      ) : (
        <>
          <label className="column">
            {t("settings.jarvis.provider")}
            <select value={provider} onChange={(e) => setProvider(e.currentTarget.value)}>
              {info.providers.map((p) => (
                <option key={p} value={p}>
                  {p === "ollama" ? t("settings.jarvis.ollama") : p}
                </option>
              ))}
            </select>
          </label>
          {online && (
            <>
              <label className="column">
                {t("settings.jarvis.model")}
                <input
                  value={model}
                  maxLength={100}
                  onChange={(e) => setModel(e.currentTarget.value)}
                />
              </label>
              {provider === "custom" && (
                <label className="column">
                  {t("settings.jarvis.base")}
                  <input
                    value={base}
                    maxLength={200}
                    onChange={(e) => setBase(e.currentTarget.value)}
                  />
                </label>
              )}
              <label className="column">
                {t("settings.jarvis.key")}:{" "}
                <strong>
                  {info.provider === provider && info.has_key
                    ? t("settings.jarvis.key_saved")
                    : t("settings.jarvis.key_missing")}
                </strong>
                <input
                  type="password"
                  autoComplete="off"
                  value={key}
                  onChange={(e) => setKey(e.currentTarget.value)}
                />
              </label>
              <p className="hint">{t("settings.jarvis.online_warning")}</p>
            </>
          )}
          <p className="hint">{t("settings.jarvis.voice_hint")}</p>
          <div className="row">
            <button type="button" disabled={check.state === "busy"} onClick={() => void apply()}>
              {t("settings.jarvis.apply")}
            </button>
            {check.state !== "idle" && (
              <span role="status">
                {check.state === "busy" ? "…" : check.state === "ok" ? "✓ " : "✗ "}
                {check.text}
              </span>
            )}
          </div>
        </>
      )}
    </fieldset>
  );
}
