import type { Palette } from "./paint";

const TOKENS: Record<string, string> = {
  todo: "--color-neutral",
  passed: "--color-good",
  passed_out: "--color-good",
  in_progress: "--color-accent",
  exam_pending: "--color-accent",
  stale_passed: "--color-warning",
  blocked: "--color-warning",
  failed: "--color-critical",
};

const PLAIN: Palette = {
  edge: "#7b8590",
  text: "#5b656f",
  ring: "#f4f5f7",
  paper: "#f4f5f7",
  status: {},
};

export function palette(host: HTMLElement): Palette {
  const probe = host.ownerDocument.createElement("span");
  probe.style.display = "none";
  host.append(probe);
  const window = host.ownerDocument.defaultView;
  if (window === null) return PLAIN;
  const seen = (token: string) => {
    probe.style.color = `var(${token})`;
    return window.getComputedStyle(probe).color || PLAIN.edge;
  };
  const made: Palette = {
    edge: seen("--color-border-strong"),
    text: seen("--color-fg-muted"),
    ring: seen("--color-bg"),
    paper: seen("--color-bg"),
    status: Object.fromEntries(
      Object.entries(TOKENS).map(([status, token]) => [status, seen(token)]),
    ),
  };
  probe.remove();
  return made;
}
