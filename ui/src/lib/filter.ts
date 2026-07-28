export function matches(title: string, needle: string): boolean {
  return title.toLocaleLowerCase().includes(needle.trim().toLocaleLowerCase());
}
