export interface Program {
  id: string;
  title: string;
  path: string;
}

export function programs(text: string): Program[] {
  const found: Program[] = [];
  let current: Partial<Program> | null = null;
  for (const line of text.split("\n")) {
    const started = line.trim().startsWith("- ");
    const body = started ? line.trim().slice(2) : line.trim();
    if (started) {
      keep(found, current);
      current = {};
    }
    if (current === null) {
      continue;
    }
    const split = body.indexOf(":");
    if (split < 0) {
      continue;
    }
    const name = body.slice(0, split).trim();
    const value = unquoted(body.slice(split + 1).trim());
    if (name === "id" || name === "title" || name === "path") {
      current[name] = value;
    }
  }
  keep(found, current);
  return found;
}

function keep(found: Program[], current: Partial<Program> | null): void {
  if (current === null) {
    return;
  }
  const { id, title, path } = current;
  if (id !== undefined && title !== undefined && path !== undefined) {
    found.push({ id, title, path });
  }
}

function unquoted(value: string): string {
  if (!value.startsWith('"') || !value.endsWith('"') || value.length < 2) {
    return value;
  }
  return value.slice(1, -1).replace(/\\"/g, '"').replace(/\\\\/g, "\\");
}
