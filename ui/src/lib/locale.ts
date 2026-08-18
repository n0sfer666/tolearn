import { LOCALES, type Locale } from "../i18n";

export const MIRROR = "tolearn.locale";

export function known(value: string | null): Locale | null {
  return LOCALES.find((locale) => locale === value) ?? null;
}

export function remember(locale: string): void {
  const settled = known(locale);
  if (settled !== null) localStorage.setItem(MIRROR, settled);
}

export function mirrored(): Locale | null {
  return known(localStorage.getItem(MIRROR));
}

export function chosen(saved: string | null, mirror: string | null, system: string): Locale {
  return known(saved) ?? known(mirror) ?? known(system.slice(0, 2)) ?? "ru";
}
