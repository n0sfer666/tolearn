const KEYS = ["program", "node", "stage"];

export function here(href: string): boolean {
  if (typeof location === "undefined") return false;
  const now = new URL(location.href);
  const aim = new URL(href, now);
  return aim.pathname === now.pathname && KEYS.every((key) => aim.searchParams.get(key) === now.searchParams.get(key));
}
