import { en } from "./en.ts";
import { type Dictionary, type Plural, ru } from "./ru.ts";

export const LOCALES = ["ru", "en"] as const;

export type Locale = (typeof LOCALES)[number];

const DICTIONARIES: Record<Locale, Dictionary> = { ru, en };

export function isLocale(value: string): value is Locale {
  return (LOCALES as readonly string[]).includes(value);
}

export function localeOf(value: string | undefined): Locale {
  if (value === undefined || !isLocale(value)) {
    throw new Error(`неизвестная локаль: ${value}`);
  }
  return value;
}

export function strings(locale: Locale): Dictionary {
  return DICTIONARIES[locale];
}

export function plural(locale: Locale, count: number, forms: Plural): string {
  const category = new Intl.PluralRules(locale).select(count);
  const form = forms[category];
  if (form === undefined) {
    throw new Error(`нет формы «${category}» для ${count} в локали ${locale}`);
  }
  return form.replace("{n}", String(count));
}

export function localized(pathname: string, locale: Locale): string {
  const rest = pathname.replace(new RegExp(`^/(?:${LOCALES.join("|")})(?=/|$)`), "");
  return `/${locale}${rest || "/"}`;
}
