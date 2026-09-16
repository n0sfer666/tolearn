import type { NodeOut } from "../ipc";

export function slot(map: NodeOut | null, stage: string): number | null {
  if (map === null) return null;
  const at = map.stages.findIndex((row) => row.id === stage);
  return at < 0 ? null : at + 1;
}
