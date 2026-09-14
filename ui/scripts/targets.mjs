import { blocks, declarations } from "./css.mjs";

const TAP = "var(--tap-target)";
const ELEMENT = /(?:^|[\s>+~(])(?:a|button|summary)(?=$|[\s\[:.#>+~),])/;
const FLOORS = ["min-height", "min-block-size"];
const AREA = /^(.*)::(?:after|before)$/;

export function parts(selector) {
  const found = [];
  let depth = 0;
  let start = 0;

  for (let at = 0; at < selector.length; at += 1) {
    if (selector[at] === "(") depth += 1;
    if (selector[at] === ")") depth -= 1;
    if (selector[at] !== "," || depth !== 0) continue;
    found.push(selector.slice(start, at).trim());
    start = at + 1;
  }

  found.push(selector.slice(start).trim());
  return found;
}

function interactive(part, carried) {
  return ELEMENT.test(part) || [...part.matchAll(/\[(data-[\w-]+)/g)].some(([, name]) => carried.has(name));
}

export function targets(sheets, carried = new Set()) {
  const lowered = [];
  const covered = new Set();

  for (const { source, file } of sheets) {
    for (const { selector, body } of blocks(source)) {
      const found = declarations(body);
      const tapped = (property) => found.some((one) => one.property === property && one.value === TAP);
      const floored = found.some(({ property, value }) => FLOORS.includes(property) && value !== TAP);

      for (const part of parts(selector)) {
        const area = part.match(AREA);
        if (area && tapped("min-inline-size") && tapped("min-block-size")) covered.add(area[1]);
        if (!area && floored && interactive(part, carried)) lowered.push({ part, file });
      }
    }
  }

  return lowered
    .filter(({ part }) => !covered.has(part))
    .map(({ part, file }) => ({
      file,
      message: `цель касания «${part}» в ${file} ниже --tap-target — нужна зона ${part}::after с min-inline-size и min-block-size: ${TAP}`,
    }));
}
