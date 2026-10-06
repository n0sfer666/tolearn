import type { Kind } from "./name";

const KEEP: Record<string, readonly string[]> = {
  "/program/": ["program", "node"],
  "/stage/": ["program", "node", "stage"],
  "/search/": ["program"],
};

export function keep(href: string): readonly string[] {
  return KEEP[href] ?? [];
}

const NAMED: Record<string, Kind> = {
  "/program/": "program",
  "/stage/": "stage",
};

export function entity(href: string): Kind | undefined {
  return NAMED[href];
}

export function route(pathname: string, locale: string): string {
  const bare = pathname.startsWith(`/${locale}`) ? pathname.slice(locale.length + 1) : pathname;
  return bare === "" ? "/" : bare;
}
