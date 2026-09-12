import type { Locale } from "../i18n";

export function landing(locale: Locale, search: string): string {
  const refused = new URLSearchParams(search).get("refused") ?? "";
  if (refused !== "") {
    return `/${locale}/?${new URLSearchParams({ refused }).toString()}`;
  }
  return `/${locale}/`;
}
