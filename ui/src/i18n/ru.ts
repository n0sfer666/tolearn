export const ru = {
  app: "tolearn",
  language: { ru: "Русский", en: "English", switch: "Язык" },
  theme: { switch: "Тема", system: "Системная", light: "Светлая", dark: "Тёмная" },
  nav: {
    programs: "Программы",
    program: "Программа",
    topic: "Тема",
    exam: "Зачёт",
    review: "Разбор",
    back: "Назад",
  },
  programs: {
    import: "Импорт",
    importLead: "Перетащите папку программы сюда или выберите её кнопкой.",
    list: "Список",
    listLead: "Программы появятся здесь после импорта.",
    choose: "Выбрать папку",
    drop: "Перетащите папку программы сюда",
    importing: "Импортирую…",
    unreachable: "папка недоступна",
    refused: "Импорт отклонён",
    merged: "Отчёт о слиянии",
    kept: "сохранено",
    added: "добавлено",
    stale: "требует ревалидации",
    orphaned: "осиротело",
    progress: "пройдено",
  },
  program: { stages: "Этапы", lead: "Лента этапов с темами и чекпойнтами." },
  exam: {
    prompt: "Промпт",
    promptLead: "Промпт копируется одним действием и вставляется в любой чат.",
    verdict: "Вердикт",
    verdictLead: "Ответ экзаменатора вставляется целиком.",
  },
  review: { gaps: "Пробелы", gapsLead: "Пробелы из вердикта связаны с вопросами темы." },
  topic: {
    outcomes: "Результаты",
    misconceptions: "Заблуждения",
    materials: "Материалы",
    practice: "Практика",
    questions: "Вопросы",
    exam: "Зачёт",
    notes: "Конспект",
  },
  status: {
    todo: "не начата",
    in_progress: "в работе",
    exam_pending: "ждёт зачёта",
    blocked: "заблокирована",
    passed: "пройдена",
    passed_out: "зачтена вне программы",
    stale_passed: "требует ревалидации",
    failed: "провалена",
  },
  counts: {
    topics: { one: "{n} тема", few: "{n} темы", many: "{n} тем", other: "{n} темы" },
    hours: { one: "{n} час", few: "{n} часа", many: "{n} часов", other: "{n} часа" },
  },
} as const;

export type Dictionary = {
  readonly [K in keyof typeof ru]: (typeof ru)[K] extends string
    ? string
    : { readonly [F in keyof (typeof ru)[K]]: (typeof ru)[K][F] extends string ? string : Plural };
};

export type Plural = {
  readonly zero?: string;
  readonly one?: string;
  readonly two?: string;
  readonly few?: string;
  readonly many?: string;
  readonly other: string;
};
