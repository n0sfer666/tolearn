import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { blocks, declarations } from "../scripts/css.mjs";
import { parts } from "../scripts/targets.mjs";
import { styles } from "../scripts/tokens.mjs";

async function rules(target) {
  const found = [];
  for (const file of await styles()) found.push(...blocks(readFileSync(file, "utf8")));
  return found
    .filter(({ selector }) => parts(selector).some((part) => part.split(/[\s>+~]+/).at(-1) === target))
    .flatMap(({ body }) => declarations(body));
}

function last(found, property) {
  return found.filter((one) => one.property === property).at(-1)?.value;
}

test("полоса прогресса снимает нативный вид и заливается акцентом во всех движках", async () => {
  assert.equal(last(await rules("progress"), "appearance"), "none");
  for (const fill of ["progress::-webkit-progress-value", "progress::-moz-progress-bar"]) {
    assert.equal(last(await rules(fill), "background"), "var(--color-accent)", fill);
  }
  assert.equal(last(await rules("progress::-webkit-progress-bar"), "background"), "var(--color-bg-subtle)");
});

test("цвет полосы не держится на accent-color, который движок игнорирует", async () => {
  assert.equal(last(await rules("progress"), "accent-color"), undefined);
});
