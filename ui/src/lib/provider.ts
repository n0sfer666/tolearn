import type { Dictionary } from "../i18n/ru";
import type { CheckedView, HttpView, PresetView, ProbedView } from "../ipc";
import { problems } from "./problems";

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
  return problems(text).get(code(error)) ?? text.provider.failed;
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

export function offered(known: PresetView, text: Dictionary): string {
  const name = preset(known.id, text);
  return known.available ? name : `${name} (${text.provider.presetLater})`;
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
