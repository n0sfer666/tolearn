import type { Dictionary } from "../i18n/ru";
import type { CheckedView, HttpView, ProbedView, ProviderView } from "../ipc";

const OLLAMA_ENDPOINT = "http://127.0.0.1:11434";
const OPENAI_ENDPOINT = "http://127.0.0.1:8080/v1";
const TEMPERATURE_TENTHS_MAX = 20;

export function spoken(http: HttpView, api: string): HttpView {
  const typical = api === "ollama" ? OLLAMA_ENDPOINT : OPENAI_ENDPOINT;
  const untouched =
    http.endpoint.trim() === "" ||
    http.endpoint.trim() === OLLAMA_ENDPOINT ||
    http.endpoint.trim() === OPENAI_ENDPOINT;
  return { ...http, api, endpoint: untouched ? typical : http.endpoint };
}

export function heat(tenths: number): string {
  return (tenths / 10).toFixed(1);
}

export function tenths(said: string): number {
  const asked = Number.parseFloat(said);
  if (Number.isNaN(asked)) return 0;
  return Math.min(Math.max(Math.round(asked * 10), 0), TEMPERATURE_TENTHS_MAX);
}

export function numbered(said: string, most: number): number {
  const asked = Number.parseInt(said, 10);
  if (Number.isNaN(asked)) return 0;
  return Math.min(Math.max(asked, 0), most);
}

export function seen(checked: CheckedView, text: Dictionary): string {
  if (checked.version === null) {
    return `${text.provider.checked} ${checked.models.join(", ")}`;
  }
  const said = `${text.provider.version} ${checked.version}`;
  if (checked.took_ms === null) return said;
  const took = (checked.took_ms / 1000).toFixed(1);
  return `${said} · ${text.provider.took} ${took} ${text.provider.seconds}`;
}

export function answered(probed: ProbedView, text: Dictionary): string {
  if (!probed.thinking) return `${text.provider.said} ${probed.said}`;
  const took = (probed.took_ms / 1000).toFixed(1);
  return `${text.provider.thinking} ${took} ${text.provider.seconds}`;
}

export function argued(args: string[], text: Dictionary): string {
  if (args.length === 0) return text.provider.argsNone;
  const shown = args.map((arg) => (arg === "" ? text.provider.argsEmpty : arg));
  return `${text.provider.argsSeen} ${args.length} — ${shown.join(" · ")}`;
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
    "provider.model-missing": text.provider.modelMissing,
    "provider.vault": text.provider.vault,
    "provider.unknown-value": text.provider.unknownValue,
    "harness.not-found": text.provider.notFound,
    "harness.failed": text.provider.harnessFailed,
    "harness.timeout": text.provider.timedOut,
    "harness.silence": text.provider.wentQuiet,
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

export function told(error: unknown): string {
  if (code(error) !== "harness.failed") return "";
  if (error instanceof Object && "message" in error && typeof error.message === "string") {
    return error.message.trim();
  }
  return "";
}

function code(error: unknown): string {
  if (error instanceof Object && "code" in error && typeof error.code === "string") {
    return error.code;
  }
  return "";
}
