export interface Job {
  claims: boolean;
  seals: boolean;
  kept: boolean;
  steps: ReadonlySet<string>;
}

export const PLANNING: Job = { claims: true, seals: false, kept: true, steps: new Set(["plan"]) };
export const REVISING: Job = { claims: true, seals: false, kept: true, steps: new Set(["revise"]) };
export const FORKING: Job = { claims: true, seals: false, kept: true, steps: new Set(["fork"]) };
export const EXAMINING: Job = { claims: true, seals: false, kept: false, steps: new Set(["exam"]) };
export const CLARIFYING: Job = { claims: true, seals: false, kept: false, steps: new Set(["clarify"]) };
export const BUILDING: Job = {
  claims: true,
  seals: true,
  kept: true,
  steps: new Set(["part", "sources", "text", "repair", "diagrams", "write"]),
};

const KINDS: ReadonlyMap<string, Job> = new Map([
  ["plan", PLANNING],
  ["revise", REVISING],
  ["fork", FORKING],
  ["start", BUILDING],
  ["next", BUILDING],
  ["regenerate", BUILDING],
]);

export function jobOf(kind: string): Job {
  return KINDS.get(kind) ?? BUILDING;
}
