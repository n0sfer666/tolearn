export interface Advised {
  lines: string[];
  why: string;
}

export interface Hints {
  open: string;
  shut: string;
  advice: string;
  apply: string;
  own: string;
  args: string[];
  presets: Record<string, Advised[]>;
  local: string[];
  remote: string[];
}
