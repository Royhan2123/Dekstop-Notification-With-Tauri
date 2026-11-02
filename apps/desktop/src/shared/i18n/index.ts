import { create } from "zustand";
import type { Language } from "../../bindings";
import en from "./en.json";
import id from "./id.json";

export type MessageKey = keyof typeof id;

// `satisfies` memaksa en.json punya kunci yang sama persis dengan id.json.
export const dictionaries = { id, en } satisfies Record<Language, Record<MessageKey, string>>;

interface LanguageStore {
  language: Language;
  setLanguage: (l: Language) => void;
}

export const useLanguageStore = create<LanguageStore>((set) => ({
  language: "id",
  setLanguage: (language) => set({ language }),
}));

/** Terjemah `key`; `{0}`, `{1}`… diganti dengan `args`. */
export function translate(language: Language, key: MessageKey, args: (string | number)[] = []) {
  return args.reduce<string>(
    (text, arg, i) => text.split(`{${i}}`).join(String(arg)),
    dictionaries[language][key],
  );
}

export function useT() {
  const language = useLanguageStore((s) => s.language);
  return (key: MessageKey, ...args: (string | number)[]) => translate(language, key, args);
}
