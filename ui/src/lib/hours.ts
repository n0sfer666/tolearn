import type { Span } from "../ipc";

export const BUDGET = 70;

export function hours(span: Span, unit: string): string {
  return `${span.min}–${span.max} ${unit}`;
}
