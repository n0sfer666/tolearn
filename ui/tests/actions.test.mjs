import assert from "node:assert/strict";
import test, { afterEach, before, beforeEach } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { bind } from "../src/lib/keys.ts";
import { browser, settled } from "./support/dom.mjs";

let Actions;
let render;
let document;
let window;
let dispose;
let clicks;

before(
  async () => {
    window = browser("https://tolearn.local/ru/program/");
    document = window.document;
    bind(document);
    ({ default: Actions } = await island("Actions", "src/components"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const SCREEN = `<header class="bar">
    <a href="/ru/" data-back>Библиотека</a>
    <h1>Программа</h1>
    <nav><a href="/ru/" data-library>Библиотека</a><a href="/ru/search/" data-search>Поиск</a><a href="/ru/settings/" data-settings>Настройки</a></nav>
  </header>
  <main>
    <input data-filter />
    <button type="button" data-action>Экспорт   в Markdown</button>
    <button type="button" data-action disabled>Недоступно</button>
    <button type="button" data-action aria-disabled="true">Занято</button>
    <div hidden><button type="button" data-action>Спрятано</button></div>
    <button type="button" data-action aria-label="Уточнить блок">?</button>
    <a href="/ru/stage/?stage=voices" data-action>Пропустить этап</a>
    <button type="button">Не объявлено</button>
  </main>`;

function screen(body) {
  document.body.innerHTML = body;
  clicks = [];
  for (const element of document.querySelectorAll("a, button")) {
    element.addEventListener("click", (event) => {
      event.preventDefault();
      clicks.push((element.getAttribute("aria-label") ?? element.textContent).replace(/\s+/g, " "));
    });
  }
  const host = document.createElement("div");
  document.body.append(host);
  dispose = render(() => Actions({ text: ru.actions }), host);
}

beforeEach(() => screen(SCREEN));

afterEach(() => dispose());

function press(key, target = document.activeElement ?? document.body, init = {}) {
  const event = new window.KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

async function summon(target) {
  target.focus();
  const event = press("k", target, { code: "KeyK", ctrlKey: true });
  await settled();
  return event;
}

function type(text) {
  const query = document.querySelector("[data-actions-query]");
  query.value = text;
  query.dispatchEvent(new window.Event("input", { bubbles: true }));
}

const layer = () => document.querySelector("[data-actions]");
const offers = () => [...document.querySelectorAll("[data-actions] [role='option']")].map((row) => row.textContent);
const selected = () => document.querySelector("[data-actions] [aria-selected='true']")?.textContent;

test("ctrl+k из поля ввода открывает слой: действия экрана, затем пункты шапки", async () => {
  const event = await summon(document.querySelector("[data-filter]"));

  assert.equal(event.defaultPrevented, true);
  assert.ok(layer() !== null, "слой не открылся");
  assert.deepEqual(offers(), ["Экспорт в Markdown", "Уточнить блок", "Пропустить этап", "Библиотека", "Поиск", "Настройки"]);
  assert.ok(document.activeElement === document.querySelector("[data-actions-query]"), "курсор не в поле слоя");
  assert.equal(selected(), "Экспорт в Markdown");
});

test("слой — диалог с именем, поле управляет списком", async () => {
  await summon(document.body);
  const query = document.querySelector("[data-actions-query]");
  const list = document.querySelector("[data-actions] [role='listbox']");

  assert.equal(layer().getAttribute("role"), "dialog");
  assert.equal(layer().getAttribute("aria-label"), ru.actions.title);
  assert.equal(query.getAttribute("role"), "combobox");
  assert.equal(query.getAttribute("aria-controls"), list.id);
  assert.equal(query.getAttribute("aria-activedescendant"), document.querySelector("[aria-selected='true']").id);
});

test("пункт шапки не повторяет крошку, ведущую туда же", async () => {
  await summon(document.body);

  assert.deepEqual(offers().filter((label) => label === "Библиотека"), ["Библиотека"]);
  assert.equal(document.querySelectorAll('body > header a[href="/ru/"]').length, 2);
});

test("текущий пункт шапки в слой не попадает", async () => {
  document.querySelector("[data-settings]").setAttribute("aria-current", "page");

  await summon(document.body);

  assert.equal(offers().includes("Настройки"), false);
});

test("ввод сужает список без учёта регистра", async () => {
  await summon(document.body);
  press("ArrowDown");

  type("НАСТ");
  await settled();
  assert.deepEqual(offers(), ["Настройки"]);
  assert.equal(selected(), "Настройки");

  type("этап");
  await settled();
  assert.deepEqual(offers(), ["Пропустить этап"]);
});

test("пустая выдача говорит об этом, а enter ничего не делает", async () => {
  await summon(document.body);

  type("квазар");
  await settled();
  press("Enter");

  assert.deepEqual(offers(), []);
  assert.equal(document.querySelector("[data-actions-none]").textContent, ru.actions.none);
  assert.deepEqual(clicks, []);
  assert.ok(layer() !== null, "слой закрылся");
});

test("стрелки ходят по кругу, enter выполняет выбранное и закрывает слой", async () => {
  const field = document.querySelector("[data-filter]");
  await summon(field);

  press("ArrowUp");
  await settled();
  assert.equal(selected(), "Настройки");

  press("ArrowDown");
  press("ArrowDown");
  press("ArrowDown");
  await settled();
  assert.equal(selected(), "Пропустить этап");

  const event = press("Enter");
  await settled();

  assert.equal(event.defaultPrevented, true);
  assert.deepEqual(clicks, ["Пропустить этап"]);
  assert.ok(layer() === null, "слой остался открытым");
  assert.ok(document.activeElement === field, "фокус не вернулся");
});

test("esc закрывает слой, возвращает фокус и не уводит с экрана", async () => {
  const field = document.querySelector("[data-filter]");
  await summon(field);

  const event = press("Escape");
  await settled();

  assert.equal(event.defaultPrevented, true);
  assert.ok(layer() === null, "слой остался открытым");
  assert.ok(document.activeElement === field, "фокус не вернулся в поле");
  assert.deepEqual(clicks, []);
});

test("esc, закрывший слой, дальше по лестнице не идёт", async () => {
  document.querySelector("main").insertAdjacentHTML("beforeend", "<details open><summary>Уточнение</summary></details>");
  await summon(document.body);

  press("Escape", document.querySelector("[data-actions] [role='listbox']"));
  await settled();

  assert.ok(layer() === null, "слой остался открытым");
  assert.equal(document.querySelector("details").hasAttribute("open"), true);
  assert.deepEqual(clicks, []);
});

test("после закрытия esc снова работает по лестнице", async () => {
  await summon(document.body);
  press("Escape");
  await settled();

  press("Escape", document.body);

  assert.deepEqual(clicks, ["Библиотека"]);
});

test("повторное сочетание закрывает слой", async () => {
  const field = document.querySelector("[data-filter]");
  await summon(field);

  press("k", document.activeElement, { code: "KeyK", ctrlKey: true });
  await settled();

  assert.ok(layer() === null, "слой остался открытым");
  assert.ok(document.activeElement === field, "фокус не вернулся");
});

test("щелчок по пункту выполняет его, щелчок мимо слоя закрывает", async () => {
  await summon(document.body);

  document.querySelector("[data-actions] [role='option']").dispatchEvent(new window.Event("pointerdown", { bubbles: true }));
  document.querySelector("[data-actions] [role='option']").click();
  await settled();
  assert.deepEqual(clicks, ["Экспорт в Markdown"]);

  await summon(document.body);
  document.querySelector("main").dispatchEvent(new window.Event("pointerdown", { bubbles: true }));
  await settled();
  assert.ok(layer() === null, "слой остался открытым");
});

test("фокус, ушедший из слоя табом, закрывает его", async () => {
  await summon(document.body);
  const query = document.querySelector("[data-actions-query]");

  query.dispatchEvent(new window.FocusEvent("focusout", { bubbles: true, relatedTarget: document.querySelector("[data-filter]") }));
  await settled();

  assert.ok(layer() === null, "слой остался открытым");
});
