export const GLYPHS = {
  todo: "○",
  in_progress: "◐",
  exam_pending: "◑",
  blocked: "⊘",
  passed: "●",
  passed_out: "◍",
  stale_passed: "◉",
  failed: "✕",
} as const;

export type Status = keyof typeof GLYPHS;
