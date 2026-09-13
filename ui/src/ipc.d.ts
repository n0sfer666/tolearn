// Сгенерировано из контракта IPC: `cargo run -p tolearn-app --example ipc-types`.
// Руками не править — тест `типы_для_ui_совпадают_с_файлом_в_репозитории` сверяет байты.

export type IpcError = { code: string; message: string };

export type Span = {
  min: number;
  max: number;
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
  query: string;
  limit: number;
};

export type SearchOut = {
  hits: HitView[];
  indexed: number;
};

export type HitView = {
  kind: string;
  program: string;
  node: string;
  node_title: string;
  stage: string;
  title: string;
  block: string;
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
  program: string;
};

export type SpeechStateOut = {
  available: boolean;
  listening: boolean;
  language: string;
};

export type SpeechStopIn = {
  program: string;
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

export type ExportIn = {
  program: string;
  node: string;
  folder: string;
};

export type ExportOut = {
  path: string;
  files: number;
};

export type PlanProgramIn = {
  request: string;
  level: string;
};

export type RevisePlanIn = {
  request: string;
  level: string;
  plan: PlanView;
  wish: string;
};

export type PlanOut = {
  plan: PlanView;
  hours: Span;
};

export type PlanView = {
  title: string;
  slug: string;
  goal: string;
  volatility: string;
  stages: PlanStageView[];
  children: PlanPartView[];
};

export type PlanStageView = {
  id: string;
  title: string;
  hours: Span;
};

export type PlanPartView = {
  title: string;
  goal: string;
  hours: Span;
};

export type StartProgramIn = {
  request: string;
  level: string;
  plan: PlanView;
};

export type StartProgramOut = {
  program: string;
  node: string;
  stage: string;
};

export type CancelGenerationIn = {
};

export type CancelGenerationOut = {
  cancelled: boolean;
};

export type GenerationStep = {
  step: string;
  state: string;
  round: number;
  of: number;
};

export type ForkIn = {
  program: string;
  node: string;
  stage: string;
};

export type ForkOut = {
  variants: VariantView[];
};

export type VariantView = {
  id: string;
  title: string;
  hours: Span;
  why: string;
  recommended: boolean;
};

export type TakeNextIn = {
  program: string;
  node: string;
  stage: string;
  choice: number;
};

export type TakeNextOut = {
  stage: string;
};

export type RegenerateStageIn = {
  program: string;
  node: string;
  stage: string;
};

export type RegenerateStageOut = {
  stage: string;
};

export type Commands = {
  export: { input: ExportIn; output: ExportOut };
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
  plan_program: { input: PlanProgramIn; output: PlanOut };
  revise_plan: { input: RevisePlanIn; output: PlanOut };
  start_program: { input: StartProgramIn; output: StartProgramOut };
  cancel_generation: { input: CancelGenerationIn; output: CancelGenerationOut };
  fork: { input: ForkIn; output: ForkOut };
  take_next: { input: TakeNextIn; output: TakeNextOut };
  regenerate_stage: { input: RegenerateStageIn; output: RegenerateStageOut };
};

export type CommandName = keyof Commands;
