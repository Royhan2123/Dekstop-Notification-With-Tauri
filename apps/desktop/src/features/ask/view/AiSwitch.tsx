import { useEffect, useState } from "react";
import { useT } from "../../../shared/i18n";
import { commands, type AiActiveDto, type AiProvider } from "../../../shared/ipc";
import { useSettingsStore } from "../../settings/store";

/** Nama merek penyedia; sama di semua bahasa. */
const NAMES: Record<AiProvider, string> = {
  claude_api: "Claude API",
  gemini: "Gemini",
  deep_seek: "DeepSeek",
  grok: "Grok",
  open_ai: "OpenAI",
  open_router: "OpenRouter",
  groq: "Groq",
  ollama: "Ollama",
  claude_cli: "Claude Code",
};

/**
 * Pemilih AI dan model langsung di kartu Tanya AI: hanya penyedia yang siap (kunci tersimpan, Ollama, Claude Code).
 * Pilihan disimpan ke setelan, jadi sama dengan yang di Setelan dan dipakai pertanyaan berikutnya.
 */
export function AiSwitch({ label }: { label: string }) {
  const t = useT();
  const settings = useSettingsStore((s) => s.settings);
  const [ready, setReady] = useState<AiProvider[]>([]);
  const [active, setActive] = useState<AiActiveDto | null>(null);
  const [models, setModels] = useState<string[]>([]);
  const provider = settings?.ai_provider;
  const chosenModel = settings?.ai_model;

  useEffect(() => {
    void commands.aiReady().then((r) => {
      if (r.status === "ok") setReady(r.data);
    });
  }, []);

  useEffect(() => {
    if (!provider) return;
    let alive = true;
    setModels([]);
    void commands.aiModels().then((r) => {
      if (alive && r.status === "ok") setModels(r.data);
    });
    return () => {
      alive = false;
    };
  }, [provider]);

  useEffect(() => {
    if (!provider) return;
    let alive = true;
    void commands.aiActive().then((a) => {
      if (alive) setActive(a);
    });
    return () => {
      alive = false;
    };
  }, [provider, chosenModel]);

  if (!settings || !active) return null;
  const providers = ready.includes(active.provider) ? ready : [active.provider, ...ready];
  const modelOptions =
    models.includes(active.model) || !active.model ? models : [active.model, ...models];

  return (
    <div className="ask-by">
      <span>{label}</span>
      <select
        aria-label={t("settings.ai.provider")}
        value={active.provider}
        onChange={(e) =>
          // Ganti penyedia mengosongkan model supaya model bawaan penyedia baru yang dipakai.
          void commands.settingsUpdate({
            ...settings,
            ai_provider: e.currentTarget.value as AiProvider,
            ai_model: "",
          })
        }
      >
        {providers.map((p) => (
          <option key={p} value={p}>
            {NAMES[p]}
          </option>
        ))}
      </select>
      {(modelOptions.length > 0 || active.model) && (
        <select
          aria-label={t("settings.ai_model")}
          value={active.model}
          onChange={(e) =>
            void commands.settingsUpdate({ ...settings, ai_model: e.currentTarget.value })
          }
        >
          {!active.model && <option value="">{t("settings.ai.model_auto")}</option>}
          {modelOptions.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </select>
      )}
    </div>
  );
}
