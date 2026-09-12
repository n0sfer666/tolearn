export function reveal(): void {
  if (typeof location === "undefined" || location.hash.length < 2) return;
  const bar = document.querySelector(".bar")?.getBoundingClientRect().height ?? 0;
  document.documentElement.style.setProperty("scroll-padding-block-start", `${bar}px`);
  document.getElementById(location.hash.slice(1))?.scrollIntoView();
}
