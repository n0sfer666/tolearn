import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";

import { DIST } from "../scripts/budget.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { SHORTCUTS } from "../src/lib/shortcuts.ts";
import { browser } from "./support/dom.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));
const SRC = path.join(UI, "src");
const LIST = path.join(SRC, "lib", "shortcuts.ts");
const TEXT = { ru, en };
const LOCALES = ["ru", "en"];
const TERMS = {
  ru: ["Карта", "Этап", "Практика", "Зачёт или пропуск", "Развилка", "Подпрограммы"],
  en: ["Map", "Stage", "Practice", "Check or skip", "Fork", "Subprograms"],
};
const APPS = ["Obsidian", "Joplin", "Zettlr"];
const LEGACY = [
  "~/.config/tolearn/registry.yaml",
  "~/.local/share/tolearn/unpacked/",
  "~/.local/share/tolearn/history/",
  "~/.local/share/tolearn/offline/",
  "~/.local/share/tolearn/search-*",
];
const LAYOUT = /^(Actions|Toasts)\./;
const COMPARED = /\bevent\.(?:key|code)(?:\.toLowerCase\(\))?\s*[!=]==?\s*["'`]/;

before(() => {
  execFileSync("pnpm", ["exec", "astro", "build"], { cwd: UI, stdio: "inherit" });
}, { timeout: 300_000 });

function source(locale) {
  return readFileSync(path.join(DIST, locale, "help", "index.html"), "utf8");
}

function page(locale) {
  const window = browser(`https://tolearn.local/${locale}/help/`);
  window.document.body.innerHTML = source(locale).match(/<body[^>]*>([\s\S]*)<\/body>/)[1];
  return window.document;
}

function texts(nodes) {
  return [...nodes].map((node) => node.textContent.trim());
}

function code() {
  return readdirSync(SRC, { recursive: true })
    .filter((file) => /\.(ts|tsx|astro)$/.test(file))
    .map((file) => path.join(SRC, file))
    .filter((file) => file !== LIST)
    .map((file) => ({ file: path.relative(UI, file), text: readFileSync(file, "utf8") }));
}

test("помощь собрана на обоих языках статической страницей без своего острова", () => {
  for (const locale of LOCALES) {
    const html = source(locale);
    const islands = [...html.matchAll(/<astro-island[^>]*component-url="([^"]*)"/g)].map(([, url]) => path.basename(url));

    assert.match(html, new RegExp(`<html lang="${locale}"`));
    assert.ok(islands.length > 0, `${locale}: не нашёл островов шаблона — проверка ослепла`);
    for (const island of islands) assert.match(island, LAYOUT, `${locale}: на странице помощи свой остров`);
    assert.equal(page(locale).querySelector("h1")?.textContent.trim(), TEXT[locale].nav.help, locale);
  }
});

test("как устроено обучение: карта, этап, практика, зачёт или пропуск, развилка, подпрограммы", () => {
  for (const locale of LOCALES) {
    const section = page(locale).querySelector('section[data-help="learning"]');

    assert.ok(section !== null, `${locale}: нет раздела об обучении`);
    assert.deepEqual(texts(section.querySelectorAll("dt")), TERMS[locale], locale);
    for (const text of texts(section.querySelectorAll("dd"))) assert.ok(text.length > 40, `${locale}: пустое объяснение`);
  }
});

test("таблица клавиш построена из общего списка сочетаний", () => {
  for (const locale of LOCALES) {
    const rows = [...page(locale).querySelectorAll('section[data-help="keys"] tr[data-shortcut]')];

    assert.deepEqual(rows.map((row) => row.dataset.shortcut), SHORTCUTS.map(({ id }) => id), locale);
    for (const [at, row] of rows.entries()) {
      const { id, label, scope } = SHORTCUTS[at];
      assert.equal(row.querySelector("kbd")?.textContent.trim(), label, `${locale}: ${id}`);
      assert.equal(row.querySelector("td")?.textContent.trim(), TEXT[locale].shortcuts[id], `${locale}: ${id}`);
      assert.equal(row.closest("table")?.dataset.scope, scope, `${locale}: ${id} не в своей таблице`);
    }
  }
});

test("сочетание для слоя в таблице подписывается под ОС, как в шапке", () => {
  for (const locale of LOCALES) {
    const chord = page(locale).querySelector('tr[data-shortcut="summon"] kbd');

    assert.ok(chord?.hasAttribute("data-chord"), `${locale}: подсказку не переподпишет keys.ts`);
  }
});

test("обработчики сравнивают клавиши только через общий список", () => {
  for (const { file, text } of code()) {
    assert.doesNotMatch(text, COMPARED, `${file}: клавиша сравнивается мимо lib/shortcuts.ts`);
  }
});

test("у каждого сочетания из списка есть обработчик", () => {
  const all = code().map(({ text }) => text).join("\n");

  for (const { id } of SHORTCUTS) {
    assert.match(all, new RegExp(`pressed\\([^)]*"${id}"\\)`), `${id}: в списке есть, обработчика нет`);
  }
});

test("заметки вне приложения: Obsidian, Joplin и Zettlr — чем подходит и как вынести выгрузку", () => {
  for (const locale of LOCALES) {
    const section = page(locale).querySelector('section[data-help="notes"]');
    const apps = [...section.querySelectorAll("[data-app]")];

    assert.deepEqual(texts(apps.map((app) => app.querySelector("h3"))), APPS, locale);
    for (const app of apps) {
      assert.ok(app.querySelector("[data-fit]")?.textContent.trim().length > 40, `${locale}: ${app.dataset.app} без «чем подходит»`);
      assert.ok(app.querySelector("[data-how]")?.textContent.trim().length > 40, `${locale}: ${app.dataset.app} без «как вынести»`);
    }
    assert.ok(section.textContent.includes(TEXT[locale].program.export), `${locale}: не названа кнопка экспорта`);
    assert.ok(section.textContent.includes("index.md"), `${locale}: не описано, что кладёт выгрузка`);
  }
});

test("данные прежней версии: где лежат и что их можно удалить руками", () => {
  for (const locale of LOCALES) {
    const section = page(locale).querySelector('section[data-help="legacy"]');

    assert.ok(section !== null, `${locale}: нет раздела о данных v1`);
    assert.deepEqual(texts(section.querySelectorAll("li code")), LEGACY, locale);
  }
});
