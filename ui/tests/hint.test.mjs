import assert from "node:assert/strict";
import test, { after, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { browser, settled } from "./support/dom.mjs";

let Hint;
let render;
let document;

before(async () => {
  ({ document } = browser("https://tolearn.local/ru/settings/"));
  ({ default: Hint } = await island("Hint", "src/components"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const alive = [];

after(() => {
  for (const dispose of alive) dispose();
});

function mount(seconds) {
  const host = document.createElement("div");
  document.body.append(host);
  const dispose = render(
    () => Hint({ label: "Подсказка", seconds, children: "как передавать аргументы" }),
    host,
  );
  alive.push(dispose);
  return host;
}

const open = (host) => host.querySelector("[data-hint-open]");
const body = (host) => host.querySelector("[data-hint-body]");
const point = (button, kind) => button.dispatchEvent(new document.defaultView.Event(kind));
const after_ms = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

test("подсказка живёт под мышью и уходит вместе с ней", async () => {
  const host = mount();
  await settled();

  assert.equal(body(host), null);
  point(open(host), "mouseenter");
  await settled();

  assert.match(body(host).textContent, /аргументы/);
  assert.equal(open(host).getAttribute("aria-expanded"), "true");

  point(open(host), "mouseleave");
  await settled();
  assert.equal(body(host), null);
});

test("клик держит подсказку после ухода мыши и гасит её сам", async () => {
  const host = mount(0.05);
  await settled();

  point(open(host), "mouseenter");
  open(host).click();
  point(open(host), "mouseleave");
  await settled();

  assert.notEqual(body(host), null);

  await after_ms(120);
  assert.equal(body(host), null);
});

test("повторный клик закрывает подсказку сразу", async () => {
  const host = mount(60);
  await settled();

  open(host).click();
  await settled();
  assert.notEqual(body(host), null);

  open(host).click();
  await settled();
  assert.equal(body(host), null);
});

test("кнопка подписана для чтения с экрана", async () => {
  const host = mount();
  await settled();

  assert.equal(open(host).getAttribute("aria-label"), "Подсказка");
  assert.equal(open(host).getAttribute("type"), "button");
});
