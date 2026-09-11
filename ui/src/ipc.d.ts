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
  title: string;
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
  program: string;
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

export type ExportIn = {
  bundle: string;
  today: string;
  path: string;
};

export type ExportOut = {
  path: string;
  bytes: number;
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

export type ExamineIn = {
  bundle: string;
  topic: string;
};

export type ExamineOut = {
  text: string;
};

export type SettingsIn = {
  save: SettingsView | null;
};

export type SettingsView = {
  disk_budget_mb: number;
  locale: string;
  theme: string;
};

export type SearchIn = {
  bundle: string;
  query: string;
  limit: number;
};

export type SearchOut = {
  hits: HitView[];
  indexed: number;
};

export type HitView = {
  kind: string;
  roadmap: string;
  topic: string;
  title: string;
  snippet: string;
};

export type ProviderIn = {
  save: ProviderView | null;
  key: string | null;
  forget: boolean;
  check: boolean;
  probe: boolean;
};

export type ProviderOut = {
  provider: ProviderView;
  has_key: boolean;
  checked: CheckedView | null;
  probed: ProbedView | null;
  presets: PresetView[];
  advised: AdviceView[];
  outdated: DriftView | null;
};

export type DriftView = {
  removed: string[];
  added: string[];
  fingerprint: string;
};

export type ProviderView = {
  enabled: boolean;
  active: string;
  journal: boolean;
  local: HttpView;
  remote: HttpView;
  harness: HarnessView;
};

export type HttpView = {
  endpoint: string;
  api: string;
  model: string;
  num_ctx: number;
  temperature_tenths: number;
};

export type HarnessView = {
  id: string;
  command: string;
  args: string[];
  timeout_secs: number;
  dismissed_advice: string | null;
};

export type PresetView = {
  id: string;
  command: string;
  args: string[];
  available: boolean;
};

export type AdviceView = {
  id: string;
  repo: string;
  gigabytes: number;
  heavy: boolean;
  installed: boolean;
};

export type CheckedView = {
  models: string[];
  version: string | null;
  took_ms: number | null;
};

export type ProbedView = {
  said: string;
  took_ms: number;
  thinking: boolean;
};

export type LlmLogIn = {
  open: boolean;
  clear: boolean;
};

export type LlmLogOut = {
  room: string;
  records: number;
};

export type SpeechStateIn = {
  bundle: string;
};

export type SpeechStateOut = {
  available: boolean;
  listening: boolean;
  language: string;
};

export type SpeechStopIn = {
  bundle: string;
};

export type SpeechStopOut = {
  text: string;
};

export type LibraryIn = {
};

export type LibraryOut = {
  programs: ShelfView[];
  refused: RefusedView[];
};

export type ShelfView = {
  uuid: string;
  title: string;
  goal: string;
  hours: Span;
  children: RowView[];
};

export type RefusedView = {
  directory: string;
  code: string;
  message: string;
};

export type RowView = {
  id: string;
  title: string;
  hours: Span;
  ready: boolean;
};

export type ImportPackageIn = {
  path: string;
};

export type ImportPackageOut = {
  uuid: string;
  title: string;
  copy_of: string | null;
};

export type NodeIn = {
  program: string;
  node: string;
};

export type NodeOut = {
  program: string;
  uuid: string;
  title: string;
  goal: string;
  level: string;
  hours: Span;
  trail: CrumbView[];
  stages: RowView[];
  children: RowView[];
};

export type CrumbView = {
  uuid: string;
  title: string;
};

export type StageIn = {
  program: string;
  node: string;
  stage: string;
};

export type StageOut = {
  program: string;
  node: string;
  node_title: string;
  id: string;
  title: string;
  blocks: BlockView[];
  practice: TaskView;
  questions: AskView[];
};

export type BlockView = {
  id: string;
  kind: string;
  text: string;
  lang: string | null;
  src: string | null;
  license: string | null;
  attribution: string | null;
  source: string | null;
};

export type TaskView = {
  task: BlockView[];
  deliverable: string;
  constraints: ClaimView[];
  acceptance: ClaimView[];
};

export type ClaimView = {
  id: string;
  claim: string;
  check: string | null;
  expect: string;
};

export type AskView = {
  id: string;
  text: string;
};

export type Commands = {
  validate: { input: ValidateIn; output: ValidateOut };
  scan: { input: ScanIn; output: ScanOut };
  program: { input: ProgramIn; output: ProgramOut };
  topic: { input: TopicIn; output: TopicOut };
  run_check: { input: RunCheckIn; output: RunCheckOut };
  set_status: { input: SetStatusIn; output: SetStatusOut };
  parse_verdict: { input: ParseVerdictIn; output: VerdictView };
  apply_verdict: { input: ApplyVerdictIn; output: ApplyVerdictOut };
  review: { input: ReviewIn; output: ReviewOut };
  prompt: { input: PromptIn; output: PromptOut };
  examine: { input: ExamineIn; output: ExamineOut };
  programs: { input: ProgramsIn; output: ProgramsOut };
  export: { input: ExportIn; output: ExportOut };
  import: { input: ImportIn; output: ImportOut };
  settings: { input: SettingsIn; output: SettingsView };
  search: { input: SearchIn; output: SearchOut };
  provider: { input: ProviderIn; output: ProviderOut };
  llm_log: { input: LlmLogIn; output: LlmLogOut };
  speech_state: { input: SpeechStateIn; output: SpeechStateOut };
  speech_start: { input: SpeechStateIn; output: SpeechStateOut };
  speech_stop: { input: SpeechStopIn; output: SpeechStopOut };
  library: { input: LibraryIn; output: LibraryOut };
  import_package: { input: ImportPackageIn; output: ImportPackageOut };
  node: { input: NodeIn; output: NodeOut };
  stage: { input: StageIn; output: StageOut };
};

export type CommandName = keyof Commands;
