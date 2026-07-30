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
