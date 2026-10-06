import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { carriers } from "../scripts/carriers.mjs";
import { blocks, declarations } from "../scripts/css.mjs";
import { parts } from "../scripts/targets.mjs";
import { styles } from "../scripts/tokens.mjs";

const SRC = fileURLToPath(new URL("../src", import.meta.url));
const LAID = /^(?:grid|flex|inline-grid|inline-flex)$/;

function laid(body) {
  return declarations(body).some(({ property, value }) => property === "display" && LAID.test(value));
}

function names(compound) {
  return [...compound.matchAll(/\[(data-[\w-]+)/g)].map(([, name]) => name);
}

test("закрытый details не раскладывается сеткой: grid и flex только при [open]", async () => {
  const carried = await carriers(SRC, ["details"]);
  const offenders = [];

  for (const file of await styles()) {
    for (const { selector, body } of blocks(readFileSync(file, "utf8"))) {
      if (!laid(body)) continue;
      for (const part of parts(selector)) {
        const subject = part.split(/[\s>+~]+/).at(-1);
        const details = /^details(?![\w-])/.test(subject) || names(subject).some((name) => carried.has(name));
        if (details && !subject.includes("[open]")) offenders.push(`${path.basename(file)}: ${part}`);
      }
    }
  }

  assert.ok(carried.has("data-chain"));
  assert.deepEqual(offenders, []);
});
