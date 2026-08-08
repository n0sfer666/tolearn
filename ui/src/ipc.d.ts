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
  unload: UnloadView;
  practice: PracticeView;
  questions: QuestionView[];
  exam: ExamView;
};

export type UnloadView = {
  state: string;
  checked_at: number | null;
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

export type PracticeIn = {
  bundle: string;
  topic: string;
  step: string;
  now: string;
};

export type PracticeOut = {
  spent_sec: number;
  left_sec: number;
  box_min: number;
  running: boolean;
  expired: boolean;
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

export type PlanIn = {
  bundle: string;
  today: string;
};

export type PlanOut = {
  weekly_hours: number;
  daily_hours: number;
  left: Span;
  unknown: number;
  soonest: AheadView;
  latest: AheadView;
};

export type AheadView = {
  days: number;
  date: string;
};

export type ExportIn = {
  bundle: string;
  today: string;
  path: string;
  directory: string | null;
};

export type ExportOut = {
  path: string;
  bytes: number;
  plaintext: boolean;
};

export type StaleIn = {
  bundle: string;
  today: string;
};

export type StaleOut = {
  topics: ExpiredView[];
  materials: AgingView[];
};

export type ExpiredView = {
  topic: string;
  title: string;
  verified_at: string;
  expired_at: string;
};

export type AgingView = {
  topic: string;
  topic_title: string;
  title: string;
  url: string;
  stale: boolean;
  delta: string | null;
  covers_version: string | null;
  pin: string;
};

export type GraphIn = {
  bundle: string;
  today: string;
};

export type GraphOut = {
  nodes: NodeView[];
};

export type NodeView = {
  id: string;
  title: string;
  status: string;
  layer: number;
  depends_on: string[];
  blocked_by: string[];
  unlocks: string[];
};

export type StatsIn = {
  bundle: string;
};

export type StatsOut = {
  attempts: number;
  enough: boolean;
  hinted: number;
  hinted_share: number;
  kinds: KindView[];
  actions: ActionView[];
  streak: StreakView;
  calibration: string[];
};

export type KindView = {
  kind: string;
  ok: number;
  partial: number;
  miss: number;
};

export type ActionView = {
  action: string;
  count: number;
};

export type StreakView = {
  longest: number;
  topic: string;
};

export type QueueIn = {
  today: string;
};

export type QueueOut = {
  due: DueView[];
};

export type DueView = {
  program: string;
  title: string;
  bundle: string;
  topic: string;
  topic_title: string;
  due: string;
  overdue: boolean;
};

export type RepeatIn = {
  bundle: string;
  topic: string;
  today: string;
};

export type RepeatOut = {
  next_review_at: string | null;
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

export type HistoryIn = {
  bundle: string;
};

export type HistoryOut = {
  versions: VersionView[];
};

export type VersionView = {
  n: number;
  saved_at: string;
  bytes: number;
};

export type HistoryDiffIn = {
  bundle: string;
  version: number;
};

export type HistoryDiffOut = {
  added: Link[];
  removed: Link[];
  rewritten: RewrittenView[];
};

export type RewrittenView = {
  id: string;
  title: string;
  changed: string[];
  demoted: boolean;
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

export type ExamineIn = {
  bundle: string;
  topic: string;
};

export type ExamineOut = {
  text: string;
};

export type ExamLineView = {
  side: string;
  text: string;
};

export type ExamStateOut = {
  open: boolean;
  stale: boolean;
  stage: string;
  asked: number;
  total: number;
  hint_ready: boolean;
  log: ExamLineView[];
  graded: AnswerView[];
  hinted: string[];
  seconds: number;
  tokens: number;
  verdict: string | null;
};

export type ExamStateIn = {
  bundle: string;
  topic: string;
};

export type ExamStartIn = {
  bundle: string;
  topic: string;
  restart: boolean;
};

export type ExamSayIn = {
  bundle: string;
  topic: string;
  text: string;
};

export type ExamFinishIn = {
  bundle: string;
  topic: string;
  today: string;
};

export type SweepPickView = {
  topic: string;
  title: string;
  due: string | null;
  overdue: boolean;
};

export type SweepLegView = {
  topic: string;
  title: string;
  asked: number;
  total: number;
  verdict: string | null;
};

export type SweepStateOut = {
  open: boolean;
  stale: boolean;
  done: boolean;
  stage: string;
  asked: number;
  total: number;
  hint_ready: boolean;
  ready: SweepPickView[];
  legs: SweepLegView[];
  log: ExamLineView[];
  seconds: number;
  tokens: number;
};

export type SweepStateIn = {
  bundle: string;
  today: string;
};

export type SweepStartIn = {
  bundle: string;
  today: string;
  topics: number;
  restart: boolean;
};

export type SweepSayIn = {
  bundle: string;
  today: string;
  text: string;
};

export type SweepSettledView = {
  topic: string;
  title: string;
  result: string;
  status: string;
};

export type SweepAcceptOut = {
  settled: SweepSettledView[];
};

export type SettingsIn = {
  save: SettingsView | null;
};

export type SettingsView = {
  disk_budget_mb: number;
  notes_directory: string | null;
  locale: string;
  theme: string;
  history_depth: number;
  history_share_percent: number;
};

export type SearchIn = {
  bundle: string;
  query: string;
  directory: string | null;
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

export type EncryptionIn = {
  enable: boolean | null;
  phrase: string | null;
};

export type EncryptionOut = {
  enabled: boolean;
  external: boolean;
};

export type FollowIn = {
  url: string;
};

export type FollowOut = {
  program: string;
  topic: string;
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
};

export type PresetView = {
  id: string;
  command: string;
  args: string[];
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

export type SaveOfflineIn = {
  bundle: string;
  topic: string | null;
  again: string | null;
};

export type SaveOfflineOut = {
  job: string;
  total: number;
};

export type OfflineStateIn = {
  job: string;
};

export type OfflineStateOut = {
  total: number;
  done: number;
  current: string;
  finished: boolean;
  cancelled: boolean;
  bytes: number;
  saved: string[];
  skipped: LeftView[];
  failed: LeftView[];
};

export type LeftView = {
  url: string;
  why: string;
};

export type StopOfflineIn = {
  job: string;
};

export type StopOfflineOut = {
  stopping: boolean;
};

export type ReadOfflineIn = {
  url: string;
};

export type ReadOfflineOut = {
  kind: string;
  title: string;
  html: string;
  text: string;
  blocks: Block[];
  path: string;
  extracted: boolean;
};

export type Block = {
  kind: string;
  level: number;
  text: string;
  src: string;
};

export type GenerateIn = {
  subject: string;
  level: string;
  weekly_hours: number;
  weeks: number | null;
  today: string;
};

export type GenerateOut = {
  job: string;
};

export type GenerateStateIn = {
  job: string;
};

export type GenerateStateOut = {
  step: string;
  total: number;
  done: number;
  attempt: number;
  rounds: number;
  retry: number;
  tries: number;
  current: string;
  waiting: boolean;
  finished: boolean;
  cancelled: boolean;
  refused: string[];
  missed: string[];
  seconds: number;
  step_seconds: number;
  chars: number;
  ticks: number;
  tail: string;
  tokens: number | null;
  summary: SummaryView | null;
};

export type SummaryView = {
  id: string;
  title: string;
  topics: number;
  hours_min: number;
  hours_max: number;
  stages: StagedView[];
};

export type StagedView = {
  n: number;
  title: string;
  topics: number;
  first: string[];
};

export type GenerateGoIn = {
  job: string;
};

export type GenerateGoOut = {
  going: boolean;
};

export type GenerateStopIn = {
  job: string;
};

export type GenerateStopOut = {
  stopping: boolean;
};

export type GenerateAcceptIn = {
  job: string;
  today: string;
};

export type GenerateDraftIn = {
  take: boolean;
  drop: boolean;
  today: string;
};

export type GenerateDraftOut = {
  draft: DraftView | null;
  job: string | null;
};

export type DraftView = {
  id: string;
  title: string;
  total: number;
  done: number;
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
  examine: { input: ExamineIn; output: ExamineOut };
  exam_state: { input: ExamStateIn; output: ExamStateOut };
  exam_start: { input: ExamStartIn; output: ExamStateOut };
  exam_say: { input: ExamSayIn; output: ExamStateOut };
  exam_hint: { input: ExamStateIn; output: ExamStateOut };
  exam_finish: { input: ExamFinishIn; output: ExamStateOut };
  sweep_state: { input: SweepStateIn; output: SweepStateOut };
  sweep_start: { input: SweepStartIn; output: SweepStateOut };
  sweep_say: { input: SweepSayIn; output: SweepStateOut };
  sweep_hint: { input: SweepStateIn; output: SweepStateOut };
  sweep_finish: { input: SweepStateIn; output: SweepStateOut };
  sweep_accept: { input: SweepStateIn; output: SweepAcceptOut };
  practice: { input: PracticeIn; output: PracticeOut };
  programs: { input: ProgramsIn; output: ProgramsOut };
  plan: { input: PlanIn; output: PlanOut };
  stale: { input: StaleIn; output: StaleOut };
  export: { input: ExportIn; output: ExportOut };
  graph: { input: GraphIn; output: GraphOut };
  stats: { input: StatsIn; output: StatsOut };
  queue: { input: QueueIn; output: QueueOut };
  repeat: { input: RepeatIn; output: RepeatOut };
  import: { input: ImportIn; output: ImportOut };
  history: { input: HistoryIn; output: HistoryOut };
  history_diff: { input: HistoryDiffIn; output: HistoryDiffOut };
  settings: { input: SettingsIn; output: SettingsView };
  search: { input: SearchIn; output: SearchOut };
  provider: { input: ProviderIn; output: ProviderOut };
  llm_log: { input: LlmLogIn; output: LlmLogOut };
  speech_state: { input: SpeechStateIn; output: SpeechStateOut };
  speech_start: { input: SpeechStateIn; output: SpeechStateOut };
  speech_stop: { input: SpeechStopIn; output: SpeechStopOut };
  encryption: { input: EncryptionIn; output: EncryptionOut };
  follow: { input: FollowIn; output: FollowOut };
  save_offline: { input: SaveOfflineIn; output: SaveOfflineOut };
  offline_state: { input: OfflineStateIn; output: OfflineStateOut };
  stop_offline: { input: StopOfflineIn; output: StopOfflineOut };
  read_offline: { input: ReadOfflineIn; output: ReadOfflineOut };
  generate: { input: GenerateIn; output: GenerateOut };
  generate_state: { input: GenerateStateIn; output: GenerateStateOut };
  generate_go: { input: GenerateGoIn; output: GenerateGoOut };
  generate_stop: { input: GenerateStopIn; output: GenerateStopOut };
  generate_accept: { input: GenerateAcceptIn; output: ImportOut };
  generate_draft: { input: GenerateDraftIn; output: GenerateDraftOut };
};

export type CommandName = keyof Commands;
