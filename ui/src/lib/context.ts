const KEEP: Record<string, readonly string[]> = {
  "/program/": ["program"],
  "/search/": ["program"],
  "/stale/": ["program"],
  "/stats/": ["program"],
  "/graph/": ["program"],
  "/topic/": ["program", "topic"],
  "/exam/": ["program", "topic"],
  "/review/": ["program", "topic"],
  "/practice/": ["program", "topic"],
  "/notes/": ["program", "topic"],
};

export function keep(href: string): readonly string[] {
  return KEEP[href] ?? [];
}

const NAMED: Record<string, "program" | "topic"> = {
  "/program/": "program",
  "/topic/": "topic",
};

export function entity(href: string): "program" | "topic" | undefined {
  return NAMED[href];
}

export function route(pathname: string, locale: string): string {
  const bare = pathname.startsWith(`/${locale}`) ? pathname.slice(locale.length + 1) : pathname;
  return bare === "" ? "/" : bare;
}
