import type { Dictionary } from "../i18n/ru";

export const GLYPHS = {
  todo: "○",
  in_progress: "◐",
  exam_pending: "◑",
  blocked: "⊘",
  passed: "●",
  passed_out: "◍",
  stale_passed: "◉",
  failed: "✕",
} as const;

export type Status = keyof typeof GLYPHS;

export const STATUSES: readonly Status[] = [
  "todo",
  "in_progress",
  "exam_pending",
  "blocked",
  "passed",
  "passed_out",
  "stale_passed",
  "failed",
];

export function known(status: string): status is Status {
  return status in GLYPHS;
}

export function label(text: Dictionary, status: string): string {
  return known(status) ? text.status[status] : status;
}
