import i18n from "i18next";
import { initReactI18next } from "react-i18next";

/**
 * Every file in /locales is a language. Adding one needs no code change: drop
 * `xx.json` there, with `language.name` set to the name of the language in
 * that language.
 */
const files = import.meta.glob<{ default: Record<string, unknown> }>("../../locales/*.json", {
  eager: true,
});

const resources: Record<string, { translation: Record<string, unknown> }> = {};
export const LANGUAGES: { code: string; name: string }[] = [];

for (const [path, module] of Object.entries(files)) {
  const code = path.replace(/^.*\//, "").replace(/\.json$/, "");
  resources[code] = { translation: module.default };
  const language = module.default.language as { name?: string } | undefined;
  LANGUAGES.push({ code, name: language?.name ?? code });
}
LANGUAGES.sort((a, b) => a.code.localeCompare(b.code));

export const FALLBACK_LANGUAGE = "en";

/**
 * The locale for dates, numbers and the first day of the week. It follows
 * the regional format of the system when the system speaks the language of
 * the app (`en-GB` for English on a British system, so 12 Oct and not Oct 12).
 * Otherwise, for example French chosen on an English system, it is the
 * language itself.
 */
export function formatLocale(language: string): string {
  const wanted = language.toLowerCase().split("-")[0];
  const system = typeof navigator === "undefined" ? [] : (navigator.languages ?? [navigator.language]);
  return system.find((tag) => tag.toLowerCase().split("-")[0] === wanted) ?? language;
}

/** The language to use for a setting: "system" or the code of a language. */
export function resolveLanguage(setting: string): string {
  if (setting !== "system" && resources[setting]) {
    return setting;
  }
  const wanted = typeof navigator === "undefined" ? [] : (navigator.languages ?? [navigator.language]);
  for (const tag of wanted) {
    const primary = tag.toLowerCase().split("-")[0];
    if (resources[primary]) {
      return primary;
    }
  }
  return FALLBACK_LANGUAGE;
}

void i18n.use(initReactI18next).init({
  resources,
  lng: resolveLanguage("system"),
  fallbackLng: FALLBACK_LANGUAGE,
  interpolation: { escapeValue: false },
  returnNull: false,
});

export default i18n;
