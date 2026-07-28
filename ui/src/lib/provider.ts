import type { Dictionary } from "../i18n/ru";

export function reason(error: unknown, text: Dictionary): string {
  const problems: Record<string, string> = {
    "provider.disabled": text.provider.disabled,
    "provider.no-key": text.provider.noKey,
    "provider.no-model": text.provider.noModel,
    "provider.unreachable": text.provider.unreachable,
    "provider.rejected": text.provider.rejected,
    "provider.answered": text.provider.answered,
    "provider.bad-answer": text.provider.badAnswer,
    "provider.vault": text.provider.vault,
  };
  return problems[code(error)] ?? text.provider.failed;
}

function code(error: unknown): string {
  if (error instanceof Object && "code" in error && typeof error.code === "string") {
    return error.code;
  }
  return "";
}
