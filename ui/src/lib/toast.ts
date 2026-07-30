export const TONES = ["ok", "warn", "error", "info"] as const;

export type Tone = (typeof TONES)[number];

export interface Said {
  tone: Tone;
  text: string;
}

export const TOASTED = "tolearn:toast";

export function toast(tone: Tone, text: string): void {
  if (text === "" || typeof window === "undefined") return;
  window.dispatchEvent(new CustomEvent(TOASTED, { detail: { tone, text } }));
}

export function said(event: Event): Said | null {
  if (!("detail" in event)) return null;
  const detail = event.detail;
  if (typeof detail !== "object" || detail === null) return null;
  if (!("tone" in detail) || !("text" in detail)) return null;
  const { tone, text } = detail;
  if (typeof text !== "string" || typeof tone !== "string" || !tuned(tone)) return null;
  return { tone, text };
}

export function explain(failure: unknown): string {
  if (typeof failure === "string") return failure;
  if (typeof failure !== "object" || failure === null) return "";
  if (!("message" in failure) || typeof failure.message !== "string") return "";
  return failure.message;
}

function tuned(value: string): value is Tone {
  return TONES.some((tone) => tone === value);
}
