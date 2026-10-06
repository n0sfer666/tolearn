import assert from "node:assert/strict";
import test, { afterEach, before, beforeEach } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { bind, shortcut } from "../src/lib/keys.ts";
import { browser, settled } from "./support/dom.mjs";

let Actions;
let render;
let document;
let window;
let dispose;
let signed;

const BAR = `<header class="bar" data-folded>
    <div class="bar-here"><a href="/ru/program/" data-back>Chiptune</a><h1>Голоса чипа</h1></div>
    <nav><a href="/ru/">Библиотека</a><a href="/ru/search/">Поиск</a></nav>
    <button type="button" data-summon>Действия <kbd data-chord>⌘K</kbd></button>
  </header>
  <main><button type="button" data-action>Уточнить блок</button></main>`;

before(
  async () => {
    window = browser("https://tolearn.local/ru/stage/");
    document = window.document;
    document.body.innerHTML = BAR;
    bind(document);
    signed = [document.querySelector("[data-chord]").textContent, document.querySelector("[data-summon]").getAttribute("aria-keyshortcuts")];
    ({ default: Actions } = await island("Actions", "src/components"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

beforeEach(() => {
  document.body.innerHTML = BAR;
  for (const link of document.querySelectorAll("a")) link.addEventListener("click", (event) => event.preventDefault());
  const host = document.createElement("div");
  document.body.append(host);
  dispose = render(() => Actions({ text: ru.actions }), host);
});

afterEach(() => dispose());

const layer = () => document.querySelector("[data-actions]");
const offers = () => [...document.querySelectorAll("[data-actions] [role='option']")].map((row) => row.textContent);

async function press(target = document.querySelector("[data-summon]")) {
  target.dispatchEvent(new window.Event("pointerdown", { bubbles: true }));
  document
    .querySelector("[data-actions-query]")
    ?.dispatchEvent(new window.FocusEvent("focusout", { bubbles: true, relatedTarget: target }));
  target.click();
  await settled();
}

test("кнопка «Действия» открывает слой: действия экрана и спрятанные пункты шапки", async () => {
  await press();

  assert.ok(layer() !== null, "слой не открылся");
  assert.deepEqual(offers(), ["Уточнить блок", "Chiptune", "Библиотека", "Поиск"]);
  assert.ok(document.activeElement === document.querySelector("[data-actions-query]"), "курсор не в поле слоя");
});

test("повторное нажатие на кнопку закрывает слой, а не открывает заново", async () => {
  await press();
  await press();

  assert.ok(layer() === null, "слой остался открытым");
});

test("нажатие на подсказку внутри кнопки тоже зовёт слой", async () => {
  await press(document.querySelector("[data-chord]"));

  assert.ok(layer() !== null, "слой не открылся");
});

test("нажатие на кнопку не уводит фокус из поля слоя", () => {
  const down = new window.MouseEvent("mousedown", { bubbles: true, cancelable: true });

  document.querySelector("[data-chord]").dispatchEvent(down);

  assert.ok(down.defaultPrevented, "WebKit снимет фокус с поля, и слой закроется до клика");
});

test("клавиатура при подключении подписывает кнопку сочетанием своей ОС", () => {
  assert.deepEqual(signed, ["Ctrl+K", "Control+K"]);
});

test("подсказка сочетания — ⌘K на macOS, Ctrl+K на других ОС", () => {
  const key = document.querySelector("[data-chord]");
  const summon = document.querySelector("[data-summon]");

  shortcut(document, true);
  assert.equal(key.textContent, "⌘K");
  assert.equal(summon.getAttribute("aria-keyshortcuts"), "Meta+K");

  shortcut(document, false);
  assert.equal(key.textContent, "Ctrl+K");
  assert.equal(summon.getAttribute("aria-keyshortcuts"), "Control+K");
});
