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
  status: string;
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
  prompt: { input: PromptIn; output: PromptOut };
};

export type CommandName = keyof Commands;
