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

function known(text: Dictionary): ReadonlyMap<string, string> {
  return new Map([
    ...problems(text),
    ["generate.busy", text.generate.busy],
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
    reason: told(failure, known(text)) || text.generate.failed,
    settings: settles(coded(failure)),
    failed: true,
  };
}
