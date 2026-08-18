import type { AdviceView } from "../ipc";

export function names(advice: AdviceView, api: string): string {
  return api === "ollama" ? advice.id : advice.repo;
}

export function installs(advice: AdviceView, api: string): string {
  if (api === "ollama") return `ollama pull ${advice.id}`;
  return `llama-server -hf ${advice.repo}`;
}
