export function reveal(): void {
  if (typeof location === "undefined" || location.hash.length < 2) return;
  document.getElementById(location.hash.slice(1))?.scrollIntoView();
}
