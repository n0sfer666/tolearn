import assert from "node:assert/strict";
import test, { after, before, beforeEach } from "node:test";

import { island } from "../scripts/island.mjs";
import { browser, settled } from "./support/dom.mjs";
import { rule } from "./support/styles.mjs";

let Hint;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/settings/"));
    ({ default: Hint } = await island("Hint", "src/components"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const alive = [];

const dropped = () => {
  for (const dispose of alive.splice(0)) dispose();
};

beforeEach(() => {
  dropped();
  document.body.innerHTML = "";
});

after(dropped);

function mount() {
  const host = document.createElement("div");
  document.body.append(host);
  const dispose = render(
    () => Hint({ label: "Подсказка", shut: "Закрыть", children: "как передавать аргументы" }),
    host,
  );
  alive.push(dispose);
  return host;
}

const opener = (host) => host.querySelector("[data-hint-open]");
const layer = () => document.querySelector("[data-hint]");
const body = () => document.querySelector("[data-hint-body]");
const closer = () => document.querySelector("[data-hint-close]");
const shade = () => document.querySelector("[data-hint-back]");

const point = (node, kind) => node.dispatchEvent(new document.defaultView.Event(kind));
const waited = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function key(node, name, shifted = false) {
  const event = new document.defaultView.KeyboardEvent("keydown", {
    key: name,
    shiftKey: shifted,
    bubbles: true,
    cancelable: true,
  });
  node.dispatchEvent(event);
  return event;
}

function clicked(node) {
  node.dispatchEvent(new document.defaultView.MouseEvent("click", { bubbles: true }));
}

async function opened() {
  const host = mount();
  await settled();
  opener(host).click();
  await settled();
  return host;
}

test("наведение подсказку не открывает — поле не уезжает из-под курсора", async () => {
  const host = mount();
  await settled();

  assert.equal(layer(), null);
  point(opener(host), "mouseenter");
  await settled();

  assert.equal(layer(), null, "подсказка открылась по наведению");
  assert.equal(opener(host).getAttribute("aria-expanded"), "false");
});

test("клик открывает подсказку модалкой с фокусом на крестике", async () => {
  const host = await opened();

  assert.equal(layer().getAttribute("role"), "dialog");
  assert.equal(layer().getAttribute("aria-modal"), "true");
  assert.equal(layer().getAttribute("aria-label"), "Подсказка");
  assert.match(body().textContent, /аргументы/);
  assert.equal(document.activeElement, closer());
  assert.equal(opener(host).getAttribute("aria-expanded"), "true");
});

test("подсказка не гаснет сама — таймера у неё нет", async () => {
  await opened();

  await waited(120);

  assert.notEqual(layer(), null, "подсказка закрылась без участия человека");
});

test("крестик закрывает подсказку и возвращает фокус на «?»", async () => {
  const host = await opened();

  clicked(closer());
  await settled();

  assert.equal(layer(), null);
  assert.equal(document.activeElement, opener(host));
});

test("Esc закрывает подсказку и не доходит до экрана", async () => {
  const host = await opened();
  const heard = [];
  const listen = () => heard.push("escape");
  document.addEventListener("keydown", listen);

  key(closer(), "Escape");
  await settled();

  assert.equal(layer(), null);
  assert.deepEqual(heard, []);
  assert.equal(document.activeElement, opener(host));
  document.removeEventListener("keydown", listen);
});

test("Tab ходит по слою и подсказку не закрывает", async () => {
  await opened();

  key(closer(), "Tab");
  assert.equal(document.activeElement, body());
  key(body(), "Tab");
  assert.equal(document.activeElement, closer());
  key(closer(), "Tab", true);
  assert.equal(document.activeElement, body());
  assert.notEqual(layer(), null, "подсказка закрылась не от Esc");
});

test("клик по затемнению закрывает подсказку, клик по самой подсказке — нет", async () => {
  await opened();

  clicked(body());
  await settled();
  assert.notEqual(layer(), null);

  clicked(shade());
  await settled();
  assert.equal(layer(), null);
});

test("подсказка лежит поверх экрана и прокручивается внутри себя", () => {
  const back = rule("hint.css", "[data-hint-back]");
  assert.match(back, /position: fixed/);
  assert.match(back, /z-index: var\(--z-overlay\)/);

  const inner = rule("hint.css", "div[data-hint-body]");
  assert.match(inner, /overflow-y: auto/);
  assert.match(inner, /max-block-size: \d+vh/, "предел высоты в процентах не разрешится");
});

test("кнопка и крестик подписаны для чтения с экрана", async () => {
  const host = await opened();

  assert.equal(opener(host).getAttribute("aria-label"), "Подсказка");
  assert.equal(opener(host).getAttribute("type"), "button");
  assert.equal(closer().getAttribute("type"), "button");
  assert.match(closer().getAttribute("aria-label") ?? closer().textContent, /Закрыть/);
});
