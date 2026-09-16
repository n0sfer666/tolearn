import type { AskView } from "../ipc";

export function graded(questions: AskView[]): boolean {
  return questions.some((ask) => ask.result !== null);
}

export function score(template: string, questions: AskView[]): string {
  const passed = questions.filter((ask) => ask.result === "ok").length;
  return template.replace("{n}", String(passed)).replace("{m}", String(questions.length));
}
