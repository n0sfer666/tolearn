import type { GenerationOutcome, GenerationWork, ReadyStage } from "../ipc";
import { nextHref, nodeHref, stageHref } from "./links";

export function launched(locale: string, work: GenerationWork): string {
  if (work.kind === "fork" || work.kind === "next") return nextHref(locale, work);
  if (work.kind === "regenerate") return stageHref(locale, work.program, work.node, work.stage);
  return `/${locale}/new/`;
}

export function landed(locale: string, ready: ReadyStage): string {
  if (ready.stage === "") return nodeHref(locale, ready.program);
  return stageHref(locale, ready.program, ready.node, ready.stage);
}

export function opened(locale: string, outcome: GenerationOutcome): string {
  return outcome.stage === null ? launched(locale, outcome.work) : landed(locale, outcome.stage);
}
