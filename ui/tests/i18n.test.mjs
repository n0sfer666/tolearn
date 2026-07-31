import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";

import { DIST, pages, route, screen } from "../scripts/budget.mjs";
import { categories, keys, missing, pluralize, plurals, sources } from "../scripts/i18n.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));
const LOCALES = ["ru", "en"];

before(() => {
  execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
}, { timeout: 300_000 });

function page(route) {
  return readFileSync(path.join(DIST, route, "index.html"), "utf8");
}

test("каждый экран собран на каждом языке", async () => {
  const built = (await pages()).map((file) => route(file));

  for (const locale of LOCALES) {
    for (const where of ["/", "/program/", "/topic/", "/exam/", "/review/"]) {
      assert.ok(built.includes(`/${locale}${where}`), `${locale}: нет страницы ${where}`);
    }
  }
});

test("страница объявляет свой язык и говорит на нём", () => {
  assert.match(page("ru/program"), /<html lang="ru"/);
  assert.match(page("en/program"), /<html lang="en"/);
  assert.match(page("ru/program"), /Этапы/);
  assert.match(page("en/program"), /Stages/);
  assert.doesNotMatch(page("en/program"), /Этапы/);
});

test("экран не уводит в другой язык — переключатель живёт в настройках", () => {
  assert.doesNotMatch(page("ru/topic"), /<a[^>]*href="\/en\//);
  assert.doesNotMatch(page("en/topic"), /<a[^>]*href="\/ru\//);
});

test("ссылка назад остаётся внутри своего языка", () => {
  assert.match(page("en/topic"), /<a href="\/en\/program\/"/);
  assert.doesNotMatch(page("en/topic"), /<a href="\/program\/"/);
});

test("выбор языка на корне запоминается, а решает ядро", () => {
  const root = page(".");

  assert.match(root, /localStorage\.setItem\("tolearn\.locale"/);
  assert.doesNotMatch(root, /localStorage\.getItem\("tolearn\.locale"\)/);
  assert.match(root, /<script type="module" src="[^"]*BootLocale[^"]*"><\/script>/);
});

test("плюрализация идёт по CLDR-категориям языка", () => {
  const topics = { one: "{n} тема", few: "{n} темы", many: "{n} тем", other: "{n} темы" };

  assert.equal(pluralize("ru", 1, topics), "1 тема");
  assert.equal(pluralize("ru", 3, topics), "3 темы");
  assert.equal(pluralize("ru", 5, topics), "5 тем");
  assert.equal(pluralize("ru", 21, topics), "21 тема");
  assert.equal(pluralize("en", 1, { one: "{n} topic", other: "{n} topics" }), "1 topic");
  assert.equal(pluralize("en", 2, { one: "{n} topic", other: "{n} topics" }), "2 topics");
});

test("отсутствующая форма множественного числа — ошибка, а не тихий other", () => {
  assert.throws(() => pluralize("ru", 5, { one: "{n} тема", other: "{n} темы" }), /many/);
});

test("экрану уезжают формы множественного числа его языка", () => {
  assert.match(page("ru/program"), /\{n\} тем&quot;/);
  assert.match(page("en/program"), /\{n\} topics&quot;/);
  assert.doesNotMatch(page("en/program"), /\{n\} тем/);
});

test("словари покрывают друг друга ключ в ключ", () => {
  const [first, ...rest] = LOCALES.map((locale) => keys(sources(locale)));

  for (const [at, other] of rest.entries()) {
    assert.deepEqual(missing(first, other), [], `${LOCALES[at + 1]}: не хватает ключей`);
    assert.deepEqual(missing(other, first), [], `${LOCALES[at + 1]}: лишние ключи`);
  }
});

test("каждая множественная строка покрывает все категории своего языка", () => {
  for (const locale of LOCALES) {
    for (const [key, forms] of plurals(sources(locale))) {
      assert.deepEqual(missing(categories(locale), forms), [], `${locale}: ${key} без формы`);
    }
  }
});

test("гейт полноты словарей краснеет на пропущенном ключе", () => {
  assert.deepEqual(missing(["a.b", "a.c"], ["a.b"]), ["a.c"]);
});

test("бюджет считается по экрану, а не по языковой ветке", () => {
  assert.equal(screen("/ru/topic/"), "/topic/");
  assert.equal(screen("/en/"), "/");
  assert.equal(screen("/"), "/");
});
