import type { Shown } from "./view.ts";

export const ru = {
  status: {
    todo: "не начата",
    in_progress: "в работе",
    exam_pending: "ждёт зачёта",
    blocked: "заблокирована",
    passed: "пройдена",
    passed_out: "зачтена вне программы",
    stale_passed: "требует ревалидации",
    failed: "провалена",
    unknown: "статус неизвестен",
  },
  plugin: {
    foreign: "заметка не привязана к теме tolearn",
    sealed: "конспекты зашифрованы: статусы видит только приложение",
    noProgram: "программа `{roadmap}` не открыта в tolearn",
    noTopic: "темы `{topic}` нет в программе `{roadmap}`",
    overdue: "пора повторить",
    open: "Открыть тему в tolearn",
    data: "Каталог данных tolearn",
    locale: "Язык плагина (ru или en)",
  },
} as const;

export const en: Dictionary = {
  status: {
    todo: "not started",
    in_progress: "in progress",
    exam_pending: "awaiting exam",
    blocked: "blocked",
    passed: "passed",
    passed_out: "passed outside the program",
    stale_passed: "needs revalidation",
    failed: "failed",
    unknown: "status unknown",
  },
  plugin: {
    foreign: "the note is not bound to a tolearn topic",
    sealed: "notes are encrypted: only the app sees the statuses",
    noProgram: "program `{roadmap}` is not open in tolearn",
    noTopic: "there is no topic `{topic}` in program `{roadmap}`",
    overdue: "due for review",
    open: "Open the topic in tolearn",
    data: "tolearn data directory",
    locale: "Plugin language (ru or en)",
  },
};

export type Dictionary = {
  readonly [K in keyof typeof ru]: { readonly [F in keyof (typeof ru)[K]]: string };
};

export function dictionary(locale: string): Dictionary {
  return locale.startsWith("ru") ? ru : en;
}

export function describe(shown: Shown, text: Dictionary): string {
  switch (shown.kind) {
    case "foreign":
      return text.plugin.foreign;
    case "sealed":
      return text.plugin.sealed;
    case "no-program":
      return text.plugin.noProgram.replace("{roadmap}", shown.roadmap);
    case "no-topic":
      return text.plugin.noTopic
        .replace("{roadmap}", shown.roadmap)
        .replace("{topic}", shown.topic);
    case "topic": {
      const named = status(shown.status, text);
      const tail = shown.overdue ? `, ${text.plugin.overdue}` : "";
      return `${shown.title} · ${shown.topic}: ${named}${tail}`;
    }
  }
}

function status(given: string, text: Dictionary): string {
  const known: Record<string, string | undefined> = text.status;
  return known[given] ?? text.status.unknown;
}
