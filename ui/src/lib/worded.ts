import type { Dictionary } from "../i18n/ru";
import type { GenerationOutcome } from "../ipc";

export function ready(outcome: GenerationOutcome, text: Dictionary): string {
  if (outcome.stage !== null) return text.generate.readyStage;
  if (outcome.fork !== null) return text.generate.readyFork;
  if (outcome.plan !== null) return text.generate.readyMap;
  return text.generate.readyRefused;
}

export function leave(kind: string, text: Dictionary): string {
  if (kind === "plan" || kind === "revise") return text.generate.leaveMap;
  if (kind === "fork") return text.generate.leaveFork;
  if (kind === "regenerate") return text.generate.leaveRegenerate;
  return text.generate.leaveStage;
}
