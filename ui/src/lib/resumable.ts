import type { GenerationStateOut, GenerationWork } from "../ipc";

export function resumable(
  state: GenerationStateOut,
  kinds: ReadonlySet<string>,
  mine: (work: GenerationWork) => boolean = () => true,
): GenerationWork | null {
  const outcome = state.outcome;
  if (state.work === null && (outcome === null || outcome.stage !== null)) return null;
  const from = state.work ?? outcome?.work ?? null;
  return from !== null && kinds.has(from.kind) && mine(from) ? from : null;
}
