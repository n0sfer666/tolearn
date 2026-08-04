import type { Dictionary } from "../i18n/ru";
import type { HttpView, ProviderView } from "../ipc";

const OLLAMA_ENDPOINT = "http://127.0.0.1:11434";
const OPENAI_ENDPOINT = "http://127.0.0.1:8080/v1";

export function spoken(http: HttpView, api: string): HttpView {
  const typical = api === "ollama" ? OLLAMA_ENDPOINT : OPENAI_ENDPOINT;
  const untouched =
    http.endpoint.trim() === "" ||
    http.endpoint.trim() === OLLAMA_ENDPOINT ||
    http.endpoint.trim() === OPENAI_ENDPOINT;
  return { ...http, api, endpoint: untouched ? typical : http.endpoint };
}

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
    "provider.unknown-value": text.provider.unknownValue,
    "harness.not-found": text.provider.notFound,
    "harness.failed": text.provider.harnessFailed,
    "harness.timeout": text.provider.timedOut,
    "harness.truncated": text.provider.truncated,
  };
  return problems[code(error)] ?? text.provider.failed;
}

export function named(provider: ProviderView, text: Dictionary): string {
  if (provider.active === "harness") {
    return preset(provider.harness.id, text) || provider.harness.command;
  }
  const http = provider.active === "remote" ? provider.remote : provider.local;
  return http.model || text.provider.model;
}

export function preset(id: string, text: Dictionary): string {
  const names: Record<string, string> = {
    claude: text.provider.presetClaude,
    opencode: text.provider.presetOpencode,
    pi: text.provider.presetPi,
    custom: text.provider.presetCustom,
  };
  return names[id] ?? id;
}

function code(error: unknown): string {
  if (error instanceof Object && "code" in error && typeof error.code === "string") {
    return error.code;
  }
  return "";
}
