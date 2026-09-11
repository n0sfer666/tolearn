import type { Span } from "../ipc";

export function hours(span: Span, unit: string): string {
  return `${span.min}–${span.max} ${unit}`;
}
