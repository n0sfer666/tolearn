import assert from "node:assert/strict";
import test, { before, beforeEach } from "node:test";

import { SUMMONED } from "../src/lib/actions.ts";
import { apple, bind, summoned } from "../src/lib/keys.ts";
import { browser } from "./support/dom.mjs";

let document;
let summons = 0;

before(() => {
  const window = browser("https://tolearn.local/ru/stage/");
  ({ document } = window);
  window.addEventListener(SUMMONED, () => {
    summons += 1;
  });
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

function press(key, target = document.body, init = {}) {
  const event = new document.defaultView.KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

function chord(init) {
  return new document.defaultView.KeyboardEvent("keydown", { key: "k", code: "KeyK", ...init });
}

test("ctrl+k зовёт слой действий и из поля ввода", () => {
  screen(`<main><textarea></textarea></main>`);
  const area = document.querySelector("textarea");
  area.focus();
  const before = summons;

  const event = press("k", area, { code: "KeyK", ctrlKey: true });

  assert.equal(summons, before + 1);
  assert.equal(event.defaultPrevented, true);
  assert.ok(document.activeElement === area, "фокус ушёл из поля");
});

test("обычная k и ctrl с другой клавишей слой не зовут", () => {
  screen(`<main><input /></main>`);
  const before = summons;

  press("k", document.body, { code: "KeyK" });
  press("j", document.body, { code: "KeyJ", ctrlKey: true });

  assert.equal(summons, before);
});

test("⌘K на macOS, Ctrl+K на других ОС", () => {
  assert.equal(summoned(chord({ metaKey: true }), true), true);
  assert.equal(summoned(chord({ ctrlKey: true }), true), false);
  assert.equal(summoned(chord({ ctrlKey: true }), false), true);
  assert.equal(summoned(chord({ metaKey: true }), false), false);
  assert.equal(summoned(chord({ ctrlKey: true, shiftKey: true }), false), false);
  assert.equal(summoned(chord({ ctrlKey: true, altKey: true }), false), false);
  assert.equal(summoned(chord({ ctrlKey: true, metaKey: true }), false), false);
});

test("сочетание узнаётся по клавише, а не по букве раскладки", () => {
  assert.equal(summoned(chord({ key: "л", ctrlKey: true }), false), true);
  assert.equal(summoned(chord({ key: "K", code: "", ctrlKey: true }), false), true);
  assert.equal(summoned(chord({ key: "k", code: "KeyL", ctrlKey: true }), false), true);
});

test("macOS узнаётся по строке агента", () => {
  assert.equal(apple("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15"), true);
  assert.equal(apple("Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X)"), true);
  assert.equal(apple("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15"), false);
  assert.equal(apple("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"), false);
  assert.equal(apple("Mozilla/5.0 (X11; Darwin arm64) HappyDOM/20.11.1"), false);
});

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
  screen(`<main><textarea></textarea><input data-filter /></main>`);
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

test("esc на вложенном экране уходит к родителю, а не в список", () => {
  document.body.innerHTML = `<header><a href="/ru/" data-back>Назад</a></header>
    <main><nav><a href="/ru/program/?program=nes&node=tools" data-up>Инструменты</a></nav></main>`;
  const events = [];
  for (const link of document.querySelectorAll("a")) {
    link.addEventListener("click", (event) => {
      event.preventDefault();
      events.push(link.getAttribute("href"));
    });
  }

  press("Escape");

  assert.deepEqual(events, ["/ru/program/?program=nes&node=tools"]);
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
