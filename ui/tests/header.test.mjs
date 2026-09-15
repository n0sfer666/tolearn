import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";

import { DIST, pages, route } from "../scripts/budget.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser } from "./support/dom.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));
const TEXT = { ru, en };
const SECTIONS = [
  ["library", "/"],
  ["search", "/search/"],
  ["settings", "/settings/"],
  ["help", "/help/"],
];
const CURRENT = { "/": "/", "/search/": "/search/", "/settings/": "/settings/", "/help/": "/help/" };
const UP = {
  "/program/": "library",
  "/settings/": "library",
  "/help/": "library",
  "/new/": "library",
  "/stage/": "program",
  "/search/": "program",
  "/next/": "stage",
};
const SETTINGS = /^\/(?:ru|en)\/settings\/$/;

before(() => {
  execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
}, { timeout: 300_000 });

async function built() {
  return (await pages()).map((file) => ({ at: route(file), html: readFileSync(file, "utf8") }));
}

async function screens() {
  const found = [];
  for (const { at, html } of await built()) {
    const [, locale, screen] = at.match(/^\/(ru|en)(\/.*)$/) ?? [];
    if (locale !== undefined) found.push({ at, locale, screen, html });
  }
  return found;
}

function header(html) {
  const window = browser("https://tolearn.local/");
  window.document.body.innerHTML = html.match(/<header[\s\S]*?<\/header>/)[0];
  return window.document.querySelector("header");
}

function islands(html) {
  return [...html.matchAll(/component-url="([^"]+)"/g)].map(([, url]) => readFileSync(path.join(DIST, url), "utf8"));
}

test("на каждом экране шапка ведёт в библиотеку, поиск, настройки и помощь", async () => {
  for (const { at, locale, html } of await screens()) {
    const links = [...header(html).querySelectorAll("nav a")].map((link) => [link.getAttribute("href"), link.textContent.trim()]);

    assert.deepEqual(links, SECTIONS.map(([key, href]) => [`/${locale}${href}`, TEXT[locale].nav[key]]), at);
  }
});

test("пункты шапки и крошка в библиотеку названы по IA", async () => {
  const LABELS = {
    ru: ["Библиотека", "Поиск", "Настройки", "Помощь"],
    en: ["Library", "Search", "Settings", "Help"],
  };
  for (const { at, locale, screen, html } of await screens()) {
    const bar = header(html);

    assert.deepEqual([...bar.querySelectorAll("nav a")].map((link) => link.textContent.trim()), LABELS[locale], at);
    if (UP[screen] === "library") assert.equal(bar.querySelector("[data-back]").textContent.trim(), LABELS[locale][0], at);
  }
});

test("текущий пункт шапки отмечен, остальные — нет", async () => {
  for (const { at, locale, screen, html } of await screens()) {
    const marked = [...header(html).querySelectorAll('nav [aria-current="page"]')].map((link) => link.getAttribute("href"));
    const expected = CURRENT[screen] === undefined ? [] : [`/${locale}${CURRENT[screen]}`];

    assert.deepEqual(marked, expected, at);
  }
});

test("крошка назад называет уровень выше", async () => {
  for (const { at, locale, screen, html } of await screens()) {
    const back = header(html).querySelector("[data-back]");
    const up = UP[screen];

    if (up === undefined) {
      assert.ok(back === null, `${at}: крошка назад у верхнего уровня`);
      continue;
    }
    assert.equal(back?.textContent.trim(), TEXT[locale].nav[up], at);
  }
});

test("на этапе шапка свёрнута до крошек «программа / этап» и кнопки «Действия»", async () => {
  for (const { at, locale, screen, html } of await screens()) {
    const bar = header(html);
    const summon = bar.querySelector("button[data-summon]");

    if (screen !== "/stage/") {
      assert.ok(!bar.hasAttribute("data-folded") && summon === null, `${at}: шапка свёрнута не на этапе`);
      continue;
    }
    assert.ok(bar.hasAttribute("data-folded"), `${at}: шапка этапа не свёрнута`);
    assert.equal(bar.querySelector("[data-back]")?.dataset.name, "program", at);
    assert.equal(bar.querySelector("h1")?.dataset.name, "stage", at);
    assert.ok(summon !== null, `${at}: нет кнопки «Действия»`);
    assert.equal(summon.getAttribute("type"), "button");
    assert.ok(summon.textContent.includes(TEXT[locale].actions.title), `${at}: кнопка не названа`);
    assert.ok(summon.querySelector("kbd[data-chord]") !== null, `${at}: нет подсказки сочетания`);
  }
});

test("«Новая программа» — действие шапки библиотеки вне пунктов разделов, других экранов оно не касается", async () => {
  for (const { at, locale, screen, html } of await screens()) {
    const create = header(html).querySelector("a[data-new]");

    if (screen !== "/") {
      assert.ok(create === null, `${at}: «Новая программа» в чужой шапке`);
      continue;
    }
    assert.ok(create !== null, `${at}: в шапке нет «Новой программы»`);
    assert.equal(create.getAttribute("href"), `/${locale}/new/`, at);
    assert.equal(create.textContent.trim(), TEXT[locale].nav.new, at);
    assert.ok(create.hasAttribute("data-action"), `${at}: действие выпало из слоя ⌘K`);
    assert.ok(create.closest("nav") === null, `${at}: действие попало в пункты разделов`);
    assert.doesNotMatch(html, /<h2/, `${at}: заголовок раздела на библиотеке`);
  }
});

test("меню программы — действие шапки экрана программы: кнопка, а за ней экспорт в слое ⌘K", async () => {
  for (const { at, locale, screen, html } of await screens()) {
    const menu = header(html).querySelector("details[data-menu]");

    if (screen !== "/program/") {
      assert.ok(menu === null, `${at}: меню программы в чужой шапке`);
      continue;
    }
    assert.ok(menu !== null, `${at}: в шапке нет меню программы`);
    assert.equal(menu.querySelector(":scope > summary").getAttribute("aria-label"), TEXT[locale].program.menu, at);
    assert.ok(menu.closest("nav") === null, `${at}: меню попало в пункты разделов`);
    const exported = menu.querySelector("[data-export]");
    assert.ok(exported !== null, `${at}: в меню нет экспорта`);
    assert.ok(exported.hasAttribute("data-action"), `${at}: экспорт выпал из слоя ⌘K`);
  }
});

test("тема и язык — только в настройках: ни в шапке, ни на библиотеке, ни на стартовом экране", async () => {
  for (const { at, html } of await built()) {
    const own = at.match(/^\/(ru|en)\//)?.[1];
    const switches = islands(html).some((source) => /data-theme-choice|data-locale/.test(source));

    if (SETTINGS.test(at)) {
      assert.ok(switches, `${at}: в настройках нет переключателей`);
      continue;
    }
    assert.ok(!switches, `${at}: остров с переключателем темы или языка`);
    assert.doesNotMatch(html, /data-theme-choice|data-locale=/, `${at}: переключатель в разметке`);
    for (const other of ["ru", "en"].filter((locale) => locale !== own)) {
      assert.doesNotMatch(html, new RegExp(`<a[^>]*href="/${other}/`), `${at}: ссылка в другой язык`);
    }
  }
});
