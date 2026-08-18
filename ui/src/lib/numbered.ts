export interface Listed {
  lead: string;
  items: string[];
}

const POINT = /(?:^|\s)(\d{1,2})[.)]\s+/g;
const LEAST = 2;

interface Mark {
  from: number;
  at: number;
}

function marks(text: string): Mark[] {
  const found: Mark[] = [];
  let awaited = 1;
  for (const match of text.matchAll(POINT)) {
    if (Number(match[1]) !== awaited) continue;
    const from = match.index ?? 0;
    found.push({ from, at: from + match[0].length });
    awaited += 1;
  }
  return found;
}

export function listed(text: string): Listed | null {
  const found = marks(text);
  if (found.length < LEAST) return null;

  const items = found.map((mark, at) => text.slice(mark.at, found[at + 1]?.from ?? text.length).trim());
  return { lead: text.slice(0, found[0].from).trim(), items };
}
