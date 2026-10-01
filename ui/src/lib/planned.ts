import type { PlanOut, PlanView } from "../ipc";

export function planned(plan: PlanView): PlanOut {
  const rows = [...plan.stages, ...plan.children];
  return {
    plan,
    hours: {
      min: rows.reduce((sum, row) => sum + row.hours.min, 0),
      max: rows.reduce((sum, row) => sum + row.hours.max, 0),
    },
  };
}
