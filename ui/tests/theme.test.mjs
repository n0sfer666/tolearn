import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";

import { DIST } from "../scripts/budget.mjs";
import { early, switcher } from "../scripts/theme.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));
const PAGES = ["", ...["ru", "en"].flatMap((locale) => ["", "program", "topic", "exam", "review"].map((screen) => path.join(locale, screen)))];

before(() => {
  execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
}, { timeout: 300_000 });

function page(route) {
  return readFileSync(path.join(DIST, route, "index.html"), "utf8");
}

function storage(stored) {
  const store = new Map(stored === undefined ? [] : [["tolearn.theme", stored]]);
  return {
    store,
    api: {
      getItem: (key) => (store.has(key) ? store.get(key) : null),
      setItem: (key, value) => store.set(key, value),
      removeItem: (key) => store.delete(key),
    },
  };
}

function button(choice) {
  return {
    dataset: { themeChoice: choice },
    attributes: {},
    clicks: [],
    setAttribute(name, value) {
      this.attributes[name] = value;
    },
    addEventListener(_event, handler) {
      this.clicks.push(handler);
    },
    click() {
      for (const handler of this.clicks) handler();
    },
  };
}

function world(stored) {
  const root = { dataset: {} };
  const buttons = ["system", "light", "dark"].map(button);
  const { store, api } = storage(stored);
  const document = {
    documentElement: root,
    querySelectorAll: (selector) => (selector === "[data-theme-choice]" ? buttons : []),
  };
  return { root, buttons, store, context: { document, localStorage: api } };
}

function run(code, stored) {
  const open = world(stored);
  runInNewContext(code, open.context);
  return open;
}

function visit(html, stored) {
  const open = world(stored);
  runInNewContext(early(html), open.context);
  runInNewContext(switcher(html), open.context);
  return open;
}

test("выбор темы применяется в <head> до разметки — на каждой странице", () => {
  for (const route of PAGES) {
    const html = page(route);
    const head = html.slice(0, html.indexOf("<body"));
    assert.ok(head.includes("tolearn.theme"), `${route || "/"}: стартовый скрипт темы не в <head>`);
    assert.ok(!/<script[^>]*\bsrc=/.test(early(html)), `${route || "/"}: скрипт темы не встроенный`);
  }
});

test("сохранённая тема ставится атрибутом до первой отрисовки", () => {
  assert.equal(run(early(page("ru")), "dark").root.dataset.theme, "dark");
  assert.equal(run(early(page("ru")), "light").root.dataset.theme, "light");
});

test("системная тема не ставит атрибут — решает color-scheme", () => {
  assert.equal(run(early(page("ru")), undefined).root.dataset.theme, undefined);
  assert.equal(run(early(page("ru")), "sepia").root.dataset.theme, undefined);
});

test("переключатель предлагает три варианта на языке страницы", () => {
  assert.match(page("ru/topic"), /Системная/);
  assert.match(page("ru/topic"), /Светлая/);
  assert.match(page("ru/topic"), /Тёмная/);
  assert.match(page("en/topic"), /System/);
  assert.doesNotMatch(page("en/topic"), /Тёмная/);

  for (const route of PAGES) {
    const found = [...page(route).matchAll(/data-theme-choice="([a-z]+)"/g)].map(([, choice]) => choice);
    assert.deepEqual(found, ["system", "light", "dark"], `${route || "/"}: не тот набор вариантов`);
  }
});

test("текущий выбор отмечен для скринридера", () => {
  const { buttons } = visit(page("ru/topic"), "dark");
  assert.deepEqual(buttons.map(({ attributes }) => attributes["aria-pressed"]), ["false", "false", "true"]);
});

test("без выбора отмечена системная", () => {
  const { buttons } = visit(page("ru/topic"), undefined);
  assert.deepEqual(buttons.map(({ attributes }) => attributes["aria-pressed"]), ["true", "false", "false"]);
});

test("выбор применяется сразу и переживает перезапуск", () => {
  const { root, buttons, store } = visit(page("ru/topic"), undefined);
  buttons[2].click();

  assert.equal(root.dataset.theme, "dark", "тема не применилась без перезагрузки");
  assert.equal(store.get("tolearn.theme"), "dark", "выбор не сохранён");
  assert.deepEqual(buttons.map(({ attributes }) => attributes["aria-pressed"]), ["false", "false", "true"]);

  const next = run(early(page("ru/topic")), store.get("tolearn.theme"));
  assert.equal(next.root.dataset.theme, "dark", "после перезапуска тема сброшена");
});

test("возврат к системной стирает и выбор, и атрибут", () => {
  const { root, buttons, store } = visit(page("ru/topic"), "light");
  assert.equal(root.dataset.theme, "light", "сохранённая тема не применилась при загрузке");
  buttons[0].click();

  assert.equal(store.has("tolearn.theme"), false, "выбор остался в хранилище");
  assert.equal(root.dataset.theme, undefined, "атрибут остался и перебивает систему");
});

test("тема не мешает языку: у страницы два независимых ключа", () => {
  const html = page("ru/topic");
  assert.ok(html.includes("tolearn.locale"));
  assert.ok(html.includes("tolearn.theme"));
});
