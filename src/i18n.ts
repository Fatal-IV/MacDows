import { useEffect, useState } from "react";
import en from "./locales/en.json";
import tr from "./locales/tr.json";
import { getSettings, onSettingsChanged, type Settings } from "./bridge";

type Dict = Record<string, string>;
const dicts: Record<string, Dict> = { en, tr };

/** Looks up a key in the language; missing keys fall back to English, then to the key itself. */
export function translate(lang: string, key: string): string {
  return dicts[lang]?.[key] ?? (en as Dict)[key] ?? key;
}

/** Live settings (initial value plus updates from the settings window). */
export function useSettings(): Settings | null {
  const [settings, setSettings] = useState<Settings | null>(null);
  useEffect(() => {
    getSettings().then(setSettings).catch(() => {});
    const un = onSettingsChanged(setSettings);
    return () => {
      un.then((f) => f());
    };
  }, []);
  return settings;
}

export function useT(): (key: string) => string {
  const s = useSettings();
  const lang = s?.language ?? "en";
  return (key) => translate(lang, key);
}

/** The app language as a locale tag for dates and numbers. */
export function useLocale(): string {
  return useSettings()?.language === "tr" ? "tr-TR" : "en-US";
}
