import zh from "./locales/zh-CN.json";
import en from "./locales/en.json";
import backendTemplates from "./locales/backend-templates.json";

export type Locale = "zh-CN" | "en";
export type LanguagePreference = "system" | Locale;
export type Preferences = { language: LanguagePreference; locale: Locale };
export const locales: Record<Locale, Record<string, string>> = {
  "zh-CN": zh,
  en,
};
export function resolveLocale(
  language: LanguagePreference,
  systemLocale: string,
): Locale {
  return language === "system"
    ? systemLocale.toLowerCase().startsWith("zh")
      ? "zh-CN"
      : "en"
    : language;
}
export function translate(
  locale: Locale,
  key: string,
  values: Record<string, string | number> = {},
): string {
  const message = Object.hasOwn(locales[locale], key)
    ? locales[locale][key]
    : Object.hasOwn(locales["zh-CN"], key)
      ? locales["zh-CN"][key]
      : key;
  // Only supplied placeholders are replaced. Runtime syntax such as ${port} stays literal.
  return message.replace(/\{(\w+)\}/g, (match, name) =>
    Object.hasOwn(values, name) ? String(values[name]) : match,
  );
}
const escapeRegex = (value: string) =>
  value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const patterns = backendTemplates.map((template) => {
  const keys: string[] = [];
  let pattern = "",
    end = 0;
  for (const match of template.matchAll(/\{(\w+)\}/g)) {
    pattern += escapeRegex(template.slice(end, match.index)) + "([\\s\\S]*?)";
    keys.push(match[1]);
    end = match.index! + match[0].length;
  }
  return {
    template,
    keys,
    regex: new RegExp("^" + pattern + escapeRegex(template.slice(end)) + "$"),
  };
});
// Presentation adapter for the engine's existing diagnostic strings. Never apply
// this to user names, commands, paths, or raw service logs. Unknown messages survive.
export function diagnostic(locale: Locale, message: string, depth = 0): string {
  if (locale === "zh-CN" || !message || depth > 4) return message;
  if (Object.hasOwn(en, message)) return translate(locale, message);
  for (const { template, keys, regex } of patterns) {
    const match = regex.exec(message);
    if (!match) continue;
    const values = Object.fromEntries(
      keys.map((key, i) => [
        key,
        ["error", "label"].includes(key)
          ? diagnostic(locale, match[i + 1], depth + 1)
          : match[i + 1],
      ]),
    );
    return translate(locale, template, values);
  }
  return message;
}
