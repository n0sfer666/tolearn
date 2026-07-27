// Сгенерировано из контракта IPC: `cargo run -p tolearn-app --example ipc-types`.
// Руками не править — тест `типы_для_ui_совпадают_с_файлом_в_репозитории` сверяет байты.

export type IpcError = { code: string; message: string };

export type ValidateIn = {
  bundle: string;
};

export type ValidateOut = {
  ok: boolean;
  violations: Violation[];
};

export type Violation = {
  code: string;
  message: string;
};

export type ScanIn = {
  bundle: string;
};

export type ScanOut = {
  root: string;
  format: string;
  topics: string[];
  absent: Absent[];
  broken: Broken[];
};

export type Absent = {
  id: string;
  stage: number;
  generated: boolean;
  file: string;
};

export type Broken = {
  id: string;
  file: string;
  message: string;
};

export type ProgramIn = {
  bundle: string;
  today: string;
};

export type ProgramOut = {
  program: Tally;
  stages: Stage[];
  topics: TopicStatus[];
};

export type Stage = {
  n: number;
  title: string;
  checkpoint: string;
  tally: Tally;
};

export type Tally = {
  done: number;
  total: number;
  stale: number;
  share: number;
  hours_done: Span;
  hours_total: Span;
};

export type Span = {
  min: number;
  max: number;
};

export type TopicStatus = {
  id: string;
  title: string;
  stage: number;
  checkpoint: boolean;
  status: string;
  hours: Span;
  blocked_by: Link[];
};

export type Link = {
  id: string;
  title: string;
};

export type TopicIn = {
  bundle: string;
  topic: string;
  today: string;
};

export type TopicOut = {
  id: string;
  title: string;
  stage: number;
  checkpoint: boolean;
  status: string;
  hours: Span;
  blocked_by: Link[];
  verified_at: string;
  outdated: boolean;
  outcomes: string[];
  misconceptions: string[];
  materials: MaterialView[];
  practice: PracticeView;
  questions: QuestionView[];
  exam: ExamView;
};

export type MaterialView = {
  title: string;
  url: string;
  kind: string;
  tier: string;
  lang: string;
  stale: boolean;
  delta: string | null;
  offline: string;
  note: string;
};

export type PracticeView = {
  kind: string;
  tier: string;
  task: string;
  deliverable: string;
  starting_point: string | null;
  fallback: string | null;
  time_box_min: number;
  smoke_checked: boolean;
  constraints: CheckView[];
  acceptance: CheckView[];
};

export type CheckView = {
  id: string;
  claim: string;
  check: string;
  expect: string;
};

export type QuestionView = {
  id: string;
  kind: string;
  text: string;
};

export type ExamView = {
  focus: string;
  artifact_required: boolean;
  max_exchanges: number;
};

export type SetStatusIn = {
  bundle: string;
  topic: string;
  status: string;
  today: string;
};

export type SetStatusOut = {
  status: string;
};

export type ProgramsIn = {
  today: string;
};

export type ProgramsOut = {
  programs: Card[];
};

export type Card = {
  id: string;
  title: string;
  path: string;
  reachable: boolean;
  opened_at: string | null;
  tally: Tally | null;
};

export type ImportIn = {
  path: string;
  today: string;
};

export type ImportOut = {
  ok: boolean;
  id: string | null;
  title: string | null;
  violations: Violation[];
  report: Merged | null;
};

export type Merged = {
  kept: string[];
  added: string[];
  orphaned: string[];
  stale: StaleTopic[];
};

export type StaleTopic = {
  id: string;
  changed: string[];
};

export type PromptIn = {
  bundle: string;
  topic: string;
};

export type PromptOut = {
  text: string;
};

export type Commands = {
  validate: { input: ValidateIn; output: ValidateOut };
  scan: { input: ScanIn; output: ScanOut };
  program: { input: ProgramIn; output: ProgramOut };
  topic: { input: TopicIn; output: TopicOut };
  set_status: { input: SetStatusIn; output: SetStatusOut };
  prompt: { input: PromptIn; output: PromptOut };
  programs: { input: ProgramsIn; output: ProgramsOut };
  import: { input: ImportIn; output: ImportOut };
};

export type CommandName = keyof Commands;
