import { known, type Status } from "../status";
import type { Dictionary } from "../../i18n/ru";

export type Word = keyof Dictionary["steps"];

export interface Step {
  readonly to: Status;
  readonly word: Word;
}

export interface Moves {
  readonly back?: Step;
  readonly next?: Step;
}

const MOVES: Record<Status, Moves> = {
  todo: { next: { to: "in_progress", word: "start" } },
  in_progress: {
    back: { to: "todo", word: "toStart" },
    next: { to: "exam_pending", word: "toExam" },
  },
  exam_pending: {
    back: { to: "in_progress", word: "toWork" },
    next: { to: "passed", word: "pass" },
  },
  passed: { back: { to: "in_progress", word: "again" } },
  stale_passed: { next: { to: "in_progress", word: "refresh" } },
  failed: { next: { to: "in_progress", word: "restart" } },
  blocked: {},
  passed_out: {},
};

export function moves(status: string): Moves {
  return known(status) ? MOVES[status] : {};
}
