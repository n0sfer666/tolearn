export function query(name: string): string {
  if (typeof location === "undefined") {
    return "";
  }
  return new URLSearchParams(location.search).get(name) ?? "";
}
