import type { PlanOut } from "../ipc";

export interface Asked {
  request: string;
  level: string;
  locale: string;
}

export interface Drawn {
  want: Asked;
  out: PlanOut;
}

export function stale(asked: Asked, drawn: Drawn | null): boolean {
  if (drawn === null) return true;
  const { want } = drawn;
  return want.request !== asked.request || want.level !== asked.level || want.locale !== asked.locale;
}
