export function appended(draft: string, heard: string): string {
  const said = heard.trim();
  if (said === "") return draft;
  if (draft.trim() === "") return said;
  return /\s$/.test(draft) ? `${draft}${said}` : `${draft} ${said}`;
}
