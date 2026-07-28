export async function copy(text: string): Promise<void> {
  if (typeof navigator === "undefined" || navigator.clipboard === undefined) {
    throw new Error("clipboard is unavailable");
  }
  await navigator.clipboard.writeText(text);
}
