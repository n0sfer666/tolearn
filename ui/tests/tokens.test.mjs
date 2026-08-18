import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { readdir } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { blocks, declarations } from "../scripts/css.mjs";
import { ratio, themes } from "../scripts/contrast.mjs";
import { SOURCE, contrasts, naming, read, styles, tokens, values } from "../scripts/tokens.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));

test("токены берутся из docs/design, копии в ui нет", async () => {
  assert.equal(path.relative(UI, SOURCE), path.join("..", "docs", "design", "tokens.css"));

  const declared = [...tokens(read()).keys()];
  for (const file of await styles()) {
    const own = [...tokens(readFileSync(file, "utf8")).keys()];
    const copied = own.filter((name) => declared.includes(name));
    assert.deepEqual(copied, [], `${path.relative(UI, file)} переобъявляет токены`);
  }
});

test("страница подключает файл токенов, а не свой набор значений", async () => {
  const layout = readFileSync(path.join(UI, "src/layouts/Base.astro"), "utf8");

  assert.match(layout, /@design\/tokens\.css/);
});

test("разбор css видит объявления внутри правил и медиазапросов", () => {
  const found = blocks("a { color: red } @media (min-width: 640px) { b { color: blue } }");

  assert.deepEqual(
    found.map(({ selector, media }) => [selector, media]),
    [
      ["a", null],
      ["b", "(min-width: 640px)"],
    ],
  );
  assert.deepEqual(declarations(found[1].body), [{ property: "color", value: "blue" }]);
});

test("конвенция именования: шкала носит префикс семейства", () => {
  const problems = naming(["--color-fg", "--color-bg", "--colorFg", "--spacing-1", "--spacing-2"]);

  assert.deepEqual(
    problems.map(({ token }) => token),
    ["--colorFg"],
  );
});

test("семейство из одного члена отвергается", () => {
  const problems = naming(["--radius-sm", "--color-fg", "--color-bg", "--spacing-1", "--spacing-2"]);

  assert.deepEqual(
    problems.map(({ token }) => token),
    ["--radius-*"],
  );
});

test("одиночные токены, поделившие первый сегмент, обязаны стать семейством", () => {
  const problems = naming(["--tap-target", "--tap-gap"]);

  assert.equal(problems.length, 1);
  assert.match(problems[0].message, /семейство/);
});

test("настоящие токены проходят конвенцию", () => {
  assert.deepEqual(naming([...tokens(read()).keys()]), []);
});

test("hex в компоненте запрещён", () => {
  const problems = values(".a { color: #fff }", "a.css");

  assert.equal(problems.length, 1);
  assert.match(problems[0].message, /hex/);
});

test("px вне токенов запрещён, брейкпойнты в медиазапросе разрешены только объявленные", () => {
  const bad = values("@media (min-width: 700px) { .a { margin: 4px } }", "a.css");

  assert.deepEqual(bad.map(({ message }) => message.slice(0, 2)).sort(), ["px", "бр"]);
  assert.deepEqual(values("@media (min-width: 1024px) { .a { margin: var(--spacing-4) } }", "a.css"), []);
});

test("outline: none без замены запрещён", () => {
  assert.equal(values(".a:focus { outline: none }", "a.css").length, 1);
  assert.deepEqual(values(".a:focus { outline: none; box-shadow: 0 0 0 var(--focus-ring) var(--color-focus) }", "a.css"), []);
});

test("стили приложения проходят линт значений", async () => {
  for (const file of await styles()) {
    const problems = values(readFileSync(file, "utf8"), path.relative(UI, file));
    assert.deepEqual(problems, [], problems.map(({ message }) => message).join("\n"));
  }
});

test("light-dark разбирается на две темы, контраст считается по WCAG", () => {
  assert.deepEqual(themes("light-dark(#ffffff, #000000)"), { light: "#ffffff", dark: "#000000" });
  assert.equal(Math.round(ratio("#ffffff", "#000000")), 21);
  assert.equal(Math.round(ratio("#ffffff", "#ffffff")), 1);
});

test("контраст держится на обеих темах и всех трёх поверхностях", () => {
  const problems = contrasts(tokens(read()));

  assert.deepEqual(problems, [], problems.map(({ message }) => message).join("\n"));
});

test("гейт контраста краснеет, когда текст сливается с фоном", () => {
  const broken = tokens(read());
  broken.set("--color-fg-muted", "light-dark(#f0f1f2, #16191d)");
  const problems = contrasts(broken);

  assert.ok(problems.length >= 2, "просевший токен не отмечен ни на светлой, ни на тёмной теме");
  assert.ok(problems.every(({ token }) => token === "--color-fg-muted"));
});

test("просадка только в тёмной ветке токена не проходит мимо гейта", () => {
  const broken = tokens(read());
  broken.set("--color-fg", "light-dark(#16191d, #1b1f23)");
  const problems = contrasts(broken);

  assert.deepEqual(
    [...new Set(problems.map(({ theme }) => theme))],
    ["dark"],
    "тёмная тема не проверяется отдельно от светлой",
  );
  assert.deepEqual([...new Set(problems.map(({ surface }) => surface))].length, 3);
});

test("тестируется каждый файл стилей, а не пустой список", async () => {
  const found = await styles();

  assert.ok(found.length > 0, "стили приложения не найдены");
  for (const entry of await readdir(path.join(UI, "src"), { recursive: true })) {
    if (entry.endsWith(".css")) {
      assert.ok(
        found.some((file) => file.endsWith(entry.split("/").join(path.sep))),
        `${entry} мимо линта`,
      );
    }
  }
});
