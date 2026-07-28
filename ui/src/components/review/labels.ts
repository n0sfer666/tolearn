import type { Dictionary } from "../../i18n/ru";

const OUTCOMES = { ok: "resultOk", partial: "resultPartial", miss: "resultMiss" } as const;

const VERDICTS = {
  pass: "verdictPass",
  partial: "verdictPartial",
  fail: "verdictFail",
  blocked: "verdictBlocked",
} as const;

type Outcome = keyof typeof OUTCOMES;
type Verdict = keyof typeof VERDICTS;

function isOutcome(value: string): value is Outcome {
  return value in OUTCOMES;
}

function isVerdict(value: string): value is Verdict {
  return value in VERDICTS;
}

export function outcome(text: Dictionary, value: string): string {
  return isOutcome(value) ? text.review[OUTCOMES[value]] : value;
}

export function verdict(text: Dictionary, value: string): string {
  return isVerdict(value) ? text.review[VERDICTS[value]] : value;
}
