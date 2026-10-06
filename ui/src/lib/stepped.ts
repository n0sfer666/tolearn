import type { Dictionary } from "../i18n/ru";
import type { GenerationStep } from "../ipc";

function labels(text: Dictionary): ReadonlyMap<string, string> {
  return new Map([
    ["plan", text.generate.stepPlan],
    ["revise", text.generate.stepRevise],
    ["fork", text.generate.stepFork],
    ["part", text.generate.stepPart],
    ["sources", text.generate.stepSources],
    ["text", text.generate.stepText],
    ["diagrams", text.generate.stepDiagrams],
    ["write", text.generate.stepWrite],
    ["exam", text.generate.stepExam],
    ["clarify", text.generate.stepClarify],
  ]);
}

export function stepped(step: GenerationStep | null, text: Dictionary): string {
  if (step === null) return text.generate.working;
  if (step.step === "repair") {
    return text.generate.stepRepair.replace("{n}", String(step.round)).replace("{of}", String(step.of));
  }
  return labels(text).get(step.step) ?? text.generate.working;
}
