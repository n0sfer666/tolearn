import type { Dictionary } from "./ru.ts";

export const en: Dictionary = {
  app: "tolearn",
  language: { ru: "Русский", en: "English", switch: "Language" },
  nav: {
    programs: "Programs",
    program: "Program",
    topic: "Topic",
    exam: "Exam",
    review: "Review",
    back: "Back",
  },
  programs: {
    import: "Import",
    importLead: "Drop a program folder here or pick it with the button.",
    list: "List",
    listLead: "Imported programs show up here.",
  },
  program: { stages: "Stages", lead: "Stages with their topics and checkpoints." },
  exam: {
    prompt: "Prompt",
    promptLead: "The prompt is copied in one action and pasted into any chat.",
    verdict: "Verdict",
    verdictLead: "The examiner's answer is pasted whole.",
  },
  review: { gaps: "Gaps", gapsLead: "Gaps from the verdict are tied to the topic's questions." },
  topic: {
    outcomes: "Outcomes",
    misconceptions: "Misconceptions",
    materials: "Materials",
    practice: "Practice",
    questions: "Questions",
    exam: "Exam",
    notes: "Notes",
  },
  status: {
    todo: "not started",
    in_progress: "in progress",
    exam_pending: "awaiting exam",
    blocked: "blocked",
    passed: "passed",
    passed_out: "passed outside the program",
    stale_passed: "needs revalidation",
    failed: "failed",
  },
  counts: {
    topics: { one: "{n} topic", other: "{n} topics" },
    hours: { one: "{n} hour", other: "{n} hours" },
  },
};
