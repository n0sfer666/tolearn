import type { Locale } from "../i18n";

export function landing(locale: Locale, search: string): string {
  const asked = new URLSearchParams(search);
  const program = asked.get("program") ?? "";
  const topic = asked.get("topic") ?? "";
  if (program !== "" && topic !== "") {
    return `/${locale}/topic/?${new URLSearchParams({ program, topic }).toString()}`;
  }
  const refused = asked.get("refused") ?? "";
  if (refused !== "") {
    return `/${locale}/?${new URLSearchParams({ refused }).toString()}`;
  }
  return `/${locale}/`;
}
