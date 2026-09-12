const KEEP: Record<string, readonly string[]> = {
  "/program/": ["program", "node"],
  "/stage/": ["program", "node", "stage"],
  "/search/": ["program"],
};

export function keep(href: string): readonly string[] {
  return KEEP[href] ?? [];
}

const NAMED: Record<string, "program"> = {
  "/program/": "program",
};

export function entity(href: string): "program" | undefined {
  return NAMED[href];
}

export function route(pathname: string, locale: string): string {
  const bare = pathname.startsWith(`/${locale}`) ? pathname.slice(locale.length + 1) : pathname;
  return bare === "" ? "/" : bare;
}
