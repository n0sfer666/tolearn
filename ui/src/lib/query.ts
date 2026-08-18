export function query(name: string): string {
  if (typeof location === "undefined") {
    return "";
  }
  return new URLSearchParams(location.search).get(name) ?? "";
}

export function opened(): string {
  const inUrl = query("program");
  if (inUrl !== "") return inUrl;
  if (typeof localStorage === "undefined") return "";
  return localStorage.getItem("tolearn.program") ?? "";
}
