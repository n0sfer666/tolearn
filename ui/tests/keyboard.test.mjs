import assert from "node:assert/strict";
import test, { before, beforeEach } from "node:test";

import { bind } from "../src/lib/keys.ts";
import { browser } from "./support/dom.mjs";

let document;

before(() => {
  ({ document } = browser("https://tolearn.local/ru/topic/"));
  bind(document);
});

function screen(body) {
  document.body.innerHTML = body;
  const events = [];
  const back = document.querySelector("[data-back]");
  if (back !== null) {
    back.addEventListener("click", (event) => {
      event.preventDefault();
      events.push(back.getAttribute("href"));
    });
  }
  return events;
}

function press(key, target = document.body) {
  const event = new document.defaultView.KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  return event;
}

beforeEach(() => {
  document.body.innerHTML = "";
});

const LIST = `<header><a href="/ru/" data-back>Назад</a></header>
  <main><input data-filter /><ul><li>тема</li></ul></main>`;

test("слэш ставит курсор в поле фильтра", () => {
  screen(LIST);

  const event = press("/");

  assert.equal(document.activeElement, document.querySelector("[data-filter]"));
  assert.equal(event.defaultPrevented, true);
});

test("слэш внутри поля ввода печатается, а не перехватывается", () => {
  screen(`<main><textarea data-verdict-input></textarea><input data-filter /></main>`);
  const area = document.querySelector("textarea");
  area.focus();

  const event = press("/", area);

  assert.equal(document.activeElement, area);
  assert.equal(event.defaultPrevented, false);
});

test("на экране без фильтра слэш молчит", () => {
  screen(`<header><a href="/ru/" data-back>Назад</a></header><main><p>нечего фильтровать</p></main>`);

  const event = press("/");

  assert.equal(event.defaultPrevented, false);
});

test("esc уходит на уровень вверх по ссылке назад", () => {
  const events = screen(LIST);

  press("Escape");

  assert.deepEqual(events, ["/ru/"]);
});

test("esc сначала закрывает раскрытый спойлер и остаётся на экране", () => {
  const events = screen(`<header><a href="/ru/" data-back>Назад</a></header>
    <main><details open><summary>Подсказка</summary><p>текст</p></details></main>`);

  press("Escape");

  assert.equal(document.querySelector("details").hasAttribute("open"), false);
  assert.deepEqual(events, []);
});

test("esc в поле ввода снимает фокус, а не уводит с экрана", () => {
  const events = screen(LIST);
  const field = document.querySelector("[data-filter]");
  field.focus();

  press("Escape", field);

  assert.notEqual(document.activeElement, field);
  assert.deepEqual(events, []);
});
