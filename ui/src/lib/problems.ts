import type { Dictionary } from "../i18n/ru";

export function problems(text: Dictionary): ReadonlyMap<string, string> {
  return new Map([
    ["provider.disabled", text.provider.disabled],
    ["provider.no-key", text.provider.noKey],
    ["provider.no-model", text.provider.noModel],
    ["provider.unreachable", text.provider.unreachable],
    ["provider.rejected", text.provider.rejected],
    ["provider.answered", text.provider.answered],
    ["provider.bad-answer", text.provider.badAnswer],
    ["provider.model-missing", text.provider.modelMissing],
    ["provider.vault", text.provider.vault],
    ["provider.unknown-value", text.provider.unknownValue],
    ["harness.not-found", text.provider.notFound],
    ["harness.failed", text.provider.harnessFailed],
    ["harness.timeout", text.provider.timedOut],
    ["harness.silence", text.provider.wentQuiet],
    ["harness.truncated", text.provider.truncated],
  ]);
}
