export interface Bound {
  roadmap: string;
  topic: string;
}

export function bound(text: string): Bound | null {
  const rest = text.startsWith("---\n") ? text.slice(4) : null;
  if (rest === null) {
    return null;
  }
  const end = rest.indexOf("\n---");
  if (end < 0) {
    return null;
  }
  const head = rest.slice(0, end);
  const roadmap = field(head, "roadmap");
  const topic = field(head, "topic");
  if (roadmap === null || topic === null) {
    return null;
  }
  return { roadmap, topic };
}

function field(head: string, name: string): string | null {
  for (const line of head.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed.startsWith(`${name}:`)) {
      continue;
    }
    const value = unquoted(trimmed.slice(name.length + 1).trim());
    return value === "" ? null : value;
  }
  return null;
}

function unquoted(value: string): string {
  const quoted =
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"));
  return quoted && value.length > 1 ? value.slice(1, -1) : value;
}
