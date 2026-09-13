import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { blocks, declarations } from "../scripts/css.mjs";
import { parts, targets } from "../scripts/targets.mjs";
import { read, styles, tokens, values } from "../scripts/tokens.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));

async function sheet() {
  const found = [];
  for (const file of await styles()) found.push(...blocks(readFileSync(file, "utf8")));
  return found;
}

function family(found, target) {
  return found
    .filter(({ selector }) => parts(selector).includes(target))
    .flatMap(({ body }) => declarations(body))
    .filter(({ property }) => property === "font-family")
    .map(({ value }) => value)
    .at(-1);
}

test("четыре размера шрифта — модульная шкала с одним шагом от 1rem", () => {
  const map = tokens(read());
  const sizes = [...map]
    .filter(([name]) => name.startsWith("--font-size-"))
    .map(([, value]) => value);

  assert.equal(sizes.length, 4, "размеров не четыре");
  assert.ok(sizes.every((value) => value.endsWith("rem")), "размер не в rem");
  assert.equal(map.get("--font-size-md"), "1rem");
  const steps = sizes
    .map((value) => Number.parseFloat(value))
    .sort((one, other) => one - other)
    .map((size, at, all) => (at === 0 ? null : size / all[at - 1]))
    .slice(1);
  assert.ok(steps.every((step) => Math.abs(step - steps[0]) < 0.005), `шаги шкалы разные: ${steps.join(", ")}`);
  assert.ok(steps[0] >= 1.125 && steps[0] <= 1.5, `шаг шкалы ${steps[0]}`);
});

test("мера строки — 65–75 знаков, цель касания — 44 px", () => {
  const map = tokens(read());
  const measure = map.get("--measure");

  assert.match(measure, /^\d+ch$/);
  assert.ok(Number.parseInt(measure, 10) >= 65 && Number.parseInt(measure, 10) <= 75, measure);
  assert.equal(map.get("--tap-target"), "44px");
});

test("длительность движения 120–200 мс в каждом объявлении", () => {
  const found = blocks(read())
    .flatMap(({ body }) => declarations(body))
    .filter(({ property }) => property === "--duration");

  assert.ok(found.length > 0);
  for (const { value } of found) {
    assert.match(value, /^\d+ms$/);
    assert.ok(Number.parseInt(value, 10) >= 120 && Number.parseInt(value, 10) <= 200, value);
  }
});

test("два локальных семейства и системный моноширинный стек", () => {
  const map = tokens(read());
  const faces = readFileSync(path.join(UI, "src/styles/fonts.css"), "utf8");

  assert.equal(map.has("--font-family-sans"), false);
  for (const [token, name, generic] of [
    ["--font-family-text", "Literata", "serif"],
    ["--font-family-ui", "Golos Text", "sans-serif"],
  ]) {
    const stack = map.get(token).split(",").map((entry) => entry.trim());
    assert.equal(stack[0], `"${name}"`, token);
    assert.equal(stack.at(-1), generic, token);
    assert.ok(faces.includes(`font-family: "${name}"`), `${name} без @font-face`);
  }
  assert.match(map.get("--font-family-mono"), /^ui-monospace,.*monospace$/);
});

test("интерфейс набран Golos Text, текст этапа и зачёта — Literata, код — моноширинным", async () => {
  const found = await sheet();

  assert.equal(family(found, "body"), "var(--font-family-ui)");
  assert.equal(family(found, ":where(button, input, textarea, select)"), "var(--font-family-ui)");
  assert.equal(family(found, ":where(summary)"), "var(--font-family-ui)");
  for (const target of ["[data-stage-title]", "[data-stage-body]", "section[data-practice]", "section[data-questions]"]) {
    assert.equal(family(found, target), "var(--font-family-text)", target);
  }
  assert.equal(family(found, "pre"), "var(--font-family-mono)");
});

test("сырой размер, семейство, интерлиньяж и шорткат font ловятся линтом", () => {
  for (const [css, token] of [
    [".a { font-size: 0.875rem }", /--font-size-/],
    [".a { font-family: Georgia, serif }", /--font-family-/],
    [".a { line-height: 1 }", /--line-height-/],
    [".a { font: 1rem/1.5 serif }", /шорткат/],
  ]) {
    const problems = values(css, "a.css");
    assert.equal(problems.length, 1, css);
    assert.match(problems[0].message, token);
  }
  assert.deepEqual(
    values(".a { font-size: var(--font-size-sm); font-family: var(--font-family-ui); line-height: inherit }", "a.css"),
    [],
  );
});

test("движение берёт --duration и живёт только под prefers-reduced-motion: no-preference", () => {
  const calm = "@media (prefers-reduced-motion: no-preference)";
  const raw = values(`${calm} { .a { transition: color 150ms ease } }`, "a.css");
  const bare = values(".a { animation: in var(--duration) var(--easing) }", "a.css");

  assert.equal(raw.length, 1);
  assert.match(raw[0].message, /--duration/);
  assert.equal(bare.length, 1);
  assert.match(bare[0].message, /no-preference/);
  assert.deepEqual(values(`${calm} { .a { transition: color var(--duration) var(--easing) } }`, "a.css"), []);
  assert.deepEqual(values(".a { transition: none }", "a.css"), []);
});

test("разбор селектора не режет запятые внутри скобок", () => {
  assert.deepEqual(parts(":where(a, b) > c, d::after"), [":where(a, b) > c", "d::after"]);
});

test("кнопка ниже цели касания без зоны ::after ловится", () => {
  const lowered = { file: "a.css", source: "button[data-x] { min-height: 0 }" };
  const area = {
    file: "b.css",
    source: "button[data-x]::after { min-inline-size: var(--tap-target); min-block-size: var(--tap-target) }",
  };

  assert.equal(targets([lowered]).length, 1);
  assert.match(targets([lowered])[0].message, /button\[data-x\]/);
  assert.deepEqual(targets([lowered, area]), []);
  assert.equal(targets([lowered, { file: "d.css", source: 'button[data-x]::after { content: "" }' }]).length, 1);
  assert.deepEqual(targets([{ file: "c.css", source: ':where(input[type="checkbox"]) { min-height: auto }' }]), []);
});

test("стили приложения держат цель касания", async () => {
  const sheets = (await styles()).map((file) => ({ file: path.relative(UI, file), source: readFileSync(file, "utf8") }));
  const problems = targets(sheets);

  assert.deepEqual(problems, [], problems.map(({ message }) => message).join("\n"));
});
