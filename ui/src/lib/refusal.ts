import type { Dictionary } from "../i18n/ru";
import { problems } from "./problems";
import { coded } from "./toast";
import { told } from "./told";

export interface Refused {
  reason: string;
  settings: boolean;
  failed: boolean;
}

const TUNED = new Set(["generate.unrepaired", "generate.verdict"]);

function holder(failure: unknown): string {
  if (typeof failure !== "object" || failure === null) return "";
  if (!("held" in failure) || typeof failure.held !== "string") return "";
  return failure.held;
}

function busy(failure: unknown, text: Dictionary): string {
  const held = new Map([
    ["generation", text.generate.busyGeneration],
    ["exam", text.generate.busyExam],
    ["clarify", text.generate.busyClarify],
  ]);
  return held.get(holder(failure)) ?? text.generate.busy;
}

function known(failure: unknown, text: Dictionary): ReadonlyMap<string, string> {
  return new Map([
    ...problems(text),
    ["generate.busy", busy(failure, text)],
    ["generate.offline", text.generate.offline],
    ["exam.empty", text.stage.blank],
    ["generate.unclear", text.generate.unclear],
    ["clarification.absent", text.stage.clarifyGone],
  ]);
}

function settles(code: string): boolean {
  return code.startsWith("provider.") || code.startsWith("harness.") || TUNED.has(code);
}

export function refusal(failure: unknown, text: Dictionary): Refused {
  return {
    reason: told(failure, known(failure, text)) || text.generate.failed,
    settings: settles(coded(failure)),
    failed: true,
  };
}
