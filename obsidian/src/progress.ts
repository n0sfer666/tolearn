export interface State {
  status: string;
  due: string | null;
}

export function state(text: string, topic: string): State | null {
  return text.trimStart().startsWith("{") ? fromJson(text, topic) : fromYaml(text, topic);
}

function fromYaml(text: string, topic: string): State | null {
  let inside = false;
  let status: string | null = null;
  let due: string | null = null;
  for (const line of text.split("\n")) {
    const indent = line.length - line.trimStart().length;
    const trimmed = line.trim();
    if (indent === 2 && trimmed.endsWith(":")) {
      if (inside) {
        break;
      }
      inside = trimmed.slice(0, -1).trim() === topic;
      continue;
    }
    if (!inside || indent !== 4) {
      continue;
    }
    const split = trimmed.indexOf(":");
    if (split < 0) {
      continue;
    }
    const name = trimmed.slice(0, split).trim();
    const value = plain(trimmed.slice(split + 1).trim());
    if (name === "status") {
      status = value;
    }
    if (name === "next_review_at") {
      due = value;
    }
  }
  return status === null ? null : { status, due };
}

function fromJson(text: string, topic: string): State | null {
  let data: unknown = null;
  try {
    data = JSON.parse(text);
  } catch {
    return null;
  }
  const topics = record(data) ? data["topics"] : null;
  const found = record(topics) ? topics[topic] : null;
  if (!record(found)) {
    return null;
  }
  const status = found["status"];
  if (typeof status !== "string") {
    return null;
  }
  const due = found["next_review_at"];
  return { status, due: typeof due === "string" ? due : null };
}

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function plain(value: string): string | null {
  const bare = value.replace(/^"(.*)"$/, "$1").replace(/^'(.*)'$/, "$1");
  return bare === "" || bare === "null" || bare === "~" ? null : bare;
}
