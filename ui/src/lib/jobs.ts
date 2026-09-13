export interface Job {
  claims: boolean;
  seals: boolean;
  steps: ReadonlySet<string>;
}

export const PLANNING: Job = { claims: false, seals: false, steps: new Set(["plan"]) };
export const REVISING: Job = { claims: false, seals: false, steps: new Set(["revise"]) };
export const FORKING: Job = { claims: true, seals: false, steps: new Set(["fork"]) };
export const BUILDING: Job = {
  claims: true,
  seals: true,
  steps: new Set(["part", "sources", "text", "repair", "diagrams", "write"]),
};
