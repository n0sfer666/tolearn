import { listed } from "./numbered";
import { type Span, spans } from "./spans";

export type Chunk =
  | { kind: "para"; spans: Span[] }
  | { kind: "head"; level: number; spans: Span[] }
  | { kind: "quote"; spans: Span[] }
  | { kind: "item"; ordered: boolean; spans: Span[] }
  | { kind: "code"; text: string };

const FENCE = /^\s*```/;
const HEAD = /^(#{1,6})\s+(.*)$/;
const QUOTE = /^>\s?(.*)$/;
const BULLET = /^[-*•]\s+(.*)$/;
const POINT = /^\d{1,2}[.)]\s+(.*)$/;

type Mark = { kind: "quote" } | { kind: "item"; ordered: boolean };

interface Begun {
  mark: Mark;
  text: string;
}

export function worded(chunk: Chunk): Span[] {
  return chunk.kind === "code" ? [] : chunk.spans;
}

function point(text: string): Chunk {
  return { kind: "item", ordered: true, spans: spans(text) };
}

function begun(line: string): Begun | null {
  const quote = QUOTE.exec(line);
  if (quote !== null) return { mark: { kind: "quote" }, text: quote[1] };
  const bullet = BULLET.exec(line);
  if (bullet !== null) return { mark: { kind: "item", ordered: false }, text: bullet[1] };
  const numbered = POINT.exec(line);
  if (numbered !== null) return { mark: { kind: "item", ordered: true }, text: numbered[1] };
  return null;
}

function paragraph(text: string): Chunk[] {
  const list = listed(text);
  if (list === null) return [{ kind: "para", spans: spans(text) }];
  const lead: Chunk[] = list.lead === "" ? [] : [{ kind: "para", spans: spans(list.lead) }];
  return [...lead, ...list.items.map(point)];
}

function built(mark: Mark | null, text: string): Chunk[] {
  if (mark === null) return paragraph(text);
  if (mark.kind === "quote") return [{ kind: "quote", spans: spans(text) }];
  return [{ kind: "item", ordered: mark.ordered, spans: spans(text) }];
}

export function rich(text: string): Chunk[] {
  const made: Chunk[] = [];
  let para: string[] = [];
  let open: Mark | null = null;
  let fenced: string[] | null = null;

  const flush = () => {
    if (para.length > 0) made.push(...built(open, para.join(" ")));
    para = [];
    open = null;
  };

  for (const raw of text.split("\n")) {
    if (fenced !== null && FENCE.test(raw)) {
      made.push({ kind: "code", text: fenced.join("\n") });
      fenced = null;
      continue;
    }
    if (fenced !== null) {
      fenced.push(raw);
      continue;
    }
    if (FENCE.test(raw)) {
      flush();
      fenced = [];
      continue;
    }
    const line = raw.trim();
    if (line === "") {
      flush();
      continue;
    }
    const head = HEAD.exec(line);
    if (head !== null) {
      flush();
      made.push({ kind: "head", level: head[1].length, spans: spans(head[2]) });
      continue;
    }
    const one = begun(line);
    if (one === null) {
      para.push(line);
      continue;
    }
    flush();
    open = one.mark;
    if (one.text !== "") para.push(one.text);
  }

  flush();
  if (fenced !== null && fenced.length > 0) made.push({ kind: "code", text: fenced.join("\n") });
  return made;
}
