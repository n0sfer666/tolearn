export type Span =
  | { kind: "plain"; text: string }
  | { kind: "code"; text: string }
  | { kind: "strong"; text: string }
  | { kind: "em"; text: string }
  | { kind: "link"; text: string; href: string };

const MARKUP = /`([^`\n]+)`|\*\*([^*\n]+)\*\*|\*([^*\n]+)\*|\[([^\]\n]+)\]\(([^\s)]+)\)/g;
const SCHEMES = ["https://", "http://"];

function made(match: RegExpMatchArray): Span {
  const [whole, code, strong, em, label, href] = match;
  if (code !== undefined) return { kind: "code", text: code };
  if (strong !== undefined) return { kind: "strong", text: strong };
  if (em !== undefined) return { kind: "em", text: em };
  if (label !== undefined && href !== undefined && outward(href)) {
    return { kind: "link", text: label, href };
  }
  return { kind: "plain", text: whole };
}

function outward(href: string): boolean {
  return SCHEMES.some((scheme) => href.startsWith(scheme));
}

export function spans(line: string): Span[] {
  const found: Span[] = [];
  let at = 0;
  for (const match of line.matchAll(MARKUP)) {
    const start = match.index ?? 0;
    if (start > at) found.push({ kind: "plain", text: line.slice(at, start) });
    found.push(made(match));
    at = start + match[0].length;
  }
  if (at < line.length) found.push({ kind: "plain", text: line.slice(at) });
  return found;
}
