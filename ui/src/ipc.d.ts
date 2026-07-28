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

export type RunCheckIn = {
  bundle: string;
  topic: string;
  check: string;
};

export type RunCheckOut = {
  id: string;
  command: string;
  expect: string;
  code: number | null;
  timed_out: boolean;
  stdout: string;
  stderr: string;
  truncated: boolean;
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

export type StampView = {
  modified_nanos: string;
  size: number;
};

export type NoteIn = {
  bundle: string;
  topic: string;
  directory: string | null;
};

export type NoteOut = {
  body: string;
  path: string | null;
  stamp: StampView | null;
};

export type SaveNoteIn = {
  bundle: string;
  topic: string;
  body: string;
  directory: string | null;
  stamp: StampView | null;
};

export type SaveNoteOut = {
  saved: boolean;
  stamp: StampView | null;
  theirs: string | null;
};

export type AnswerView = {
  id: string;
  outcome: string;
  missed: string[];
};

export type VerdictView = {
  result: string;
  status: string;
  gaps: string[];
  notes: string[];
  missing: string[];
  unknown_questions: string[];
  failed_checks: string[];
  per_question: AnswerView[];
  hinted: boolean;
  practice_accepted: boolean;
  next_action: string | null;
  retry_after_days: number | null;
};

export type ParseVerdictIn = {
  bundle: string;
  topic: string;
  text: string;
};

export type ApplyVerdictIn = {
  bundle: string;
  topic: string;
  text: string;
  today: string;
};

export type ApplyVerdictOut = {
  status: string;
  gaps: string[];
  retry: string[];
  split_suggested: boolean;
};

export type ReviewedView = {
  id: string;
  kind: string;
  text: string;
  outcome: string | null;
  missed: string[];
};

export type PastView = {
  at: string;
  verdict: string;
};

export type ReviewIn = {
  bundle: string;
  topic: string;
};

export type ReviewOut = {
  id: string;
  title: string;
  questions: ReviewedView[];
  loose: string[];
  last: PastView | null;
  history: PastView[];
  split_suggested: boolean;
  split_request: string;
};

export type PromptIn = {
  bundle: string;
  topic: string;
};

export type PromptOut = {
  text: string;
};

export type SettingsIn = {
  save: SettingsView | null;
};

export type SettingsView = {
  disk_budget_mb: number;
  notes_directory: string | null;
  locale: string;
  theme: string;
};

export type Commands = {
  validate: { input: ValidateIn; output: ValidateOut };
  scan: { input: ScanIn; output: ScanOut };
  program: { input: ProgramIn; output: ProgramOut };
  topic: { input: TopicIn; output: TopicOut };
  run_check: { input: RunCheckIn; output: RunCheckOut };
  set_status: { input: SetStatusIn; output: SetStatusOut };
  note: { input: NoteIn; output: NoteOut };
  save_note: { input: SaveNoteIn; output: SaveNoteOut };
  parse_verdict: { input: ParseVerdictIn; output: VerdictView };
  apply_verdict: { input: ApplyVerdictIn; output: ApplyVerdictOut };
  review: { input: ReviewIn; output: ReviewOut };
  prompt: { input: PromptIn; output: PromptOut };
  programs: { input: ProgramsIn; output: ProgramsOut };
  import: { input: ImportIn; output: ImportOut };
  settings: { input: SettingsIn; output: SettingsView };
};

export type CommandName = keyof Commands;
