export function remember(theme: string): void {
  const root = document.documentElement;
  if (theme === "light" || theme === "dark") {
    localStorage.setItem("tolearn.theme", theme);
    root.dataset.theme = theme;
    return;
  }
  localStorage.removeItem("tolearn.theme");
  delete root.dataset.theme;
}
