import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { blocks } from "../../scripts/css.mjs";

const STYLES = fileURLToPath(new URL("../../src/styles/", import.meta.url));

function sheet(file) {
  return blocks(readFileSync(path.join(STYLES, file), "utf8"));
}

export function rule(file, selector) {
  const found = sheet(file).find((block) => block.selector === selector);
  if (found === undefined) throw new Error(`в ${file} нет правила «${selector}»`);
  return found.body;
}

export function rules(selector) {
  return readdirSync(STYLES)
    .filter((file) => file.endsWith(".css"))
    .flatMap((file) => sheet(file).map((block) => ({ file, ...block })))
    .filter(({ selector: own }) => own === selector || own.endsWith(` ${selector}`));
}
