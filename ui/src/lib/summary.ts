import type { SummaryView } from "../ipc";

export function summary(template: string, view: SummaryView): string {
  return template
    .replace("{n}", String(view.passed))
    .replace("{of}", String(view.total))
    .replace("{skipped}", String(view.skipped));
}
