export interface Spliced {
  text: string;
  caret: number;
}

const OPEN_TAIL = /^[\s.,;:!?)\]»]/;

export function spliced(value: string, start: number, end: number, heard: string): Spliced {
  const said = heard.trim();
  if (said === "") return { text: value, caret: end };
  const head = value.slice(0, start);
  const tail = value.slice(end);
  const lead = head === "" || /\s$/.test(head) ? "" : " ";
  const trail = tail === "" || OPEN_TAIL.test(tail) ? "" : " ";
  return {
    text: `${head}${lead}${said}${trail}${tail}`,
    caret: head.length + lead.length + said.length,
  };
}
