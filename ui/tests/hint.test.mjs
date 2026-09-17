import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test, { after, before } from "node:test";
import { fileURLToPath } from "node:url";

import { island } from "../scripts/island.mjs";
import { browser, settled } from "./support/dom.mjs";

const UI = fileURLToPath(new URL("..", import.meta.url));

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

const pressed = (node, kind) =>
  node.dispatchEvent(new document.defaultView.Event(kind, { bubbles: true }));

function key(name) {
  const event = new document.defaultView.Event("keydown", { bubbles: true });
  Object.defineProperty(event, "key", { value: name });
  document.body.dispatchEvent(event);
}

const escape = () => key("Escape");

test("наведение подсказку не открывает — поле не уезжает из-под курсора", async () => {
  const host = mount();
  await settled();

  assert.equal(body(host), null);
  point(open(host), "mouseenter");
  await settled();

  assert.equal(body(host), null, "подсказка открылась по наведению");
  assert.equal(open(host).getAttribute("aria-expanded"), "false");
});

test("клик открывает подсказку и гасит её сам", async () => {
  const host = mount(0.05);
  await settled();

  open(host).click();
  await settled();

  assert.match(body(host).textContent, /аргументы/);
  assert.equal(open(host).getAttribute("aria-expanded"), "true");

  await after_ms(120);
  assert.equal(body(host), null);
});

test("подсказка лежит поверх содержимого, а не в потоке формы", () => {
  const css = readFileSync(path.join(UI, "src/styles/hint.css"), "utf8");
  const body = css.slice(css.indexOf("span[data-hint-body] {"));
  const rules = body.slice(0, body.indexOf("}"));

  assert.match(rules, /position: absolute/);
  assert.match(rules, /z-index: var\(--z-popover\)/);
  assert.match(rules, /overflow-y: auto/);
  assert.equal(/flex: 1 0 100%/.test(rules), false, "подсказка осталась элементом потока");

  const label = css.slice(0, css.indexOf("button[data-hint-open]"));
  assert.match(label, /position: relative/, "у подписи нет своей системы координат");
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

test("Esc закрывает подсказку и не доходит до экрана", async () => {
  const host = mount(60);
  await settled();

  const heard = [];
  const listen = () => heard.push("escape");
  document.addEventListener("keydown", listen);

  open(host).click();
  await settled();
  assert.notEqual(body(host), null);

  escape();
  await settled();
  assert.equal(body(host), null);
  assert.deepEqual(heard, []);

  escape();
  document.removeEventListener("keydown", listen);
  assert.deepEqual(heard, ["escape"]);
});

test("другая клавиша закреплённую подсказку не закрывает", async () => {
  const host = mount(60);
  await settled();

  open(host).click();
  await settled();
  key("Tab");
  await settled();

  assert.ok(body(host) !== null, "подсказка закрылась не от Esc");
  escape();
});

test("клик мимо закрывает подсказку, клик внутри — нет", async () => {
  const host = mount(60);
  await settled();

  open(host).click();
  await settled();

  pressed(body(host), "pointerdown");
  await settled();
  assert.notEqual(body(host), null);

  pressed(document.body, "pointerdown");
  await settled();
  assert.equal(body(host), null);
});

test("кнопка подписана для чтения с экрана", async () => {
  const host = mount();
  await settled();

  assert.equal(open(host).getAttribute("aria-label"), "Подсказка");
  assert.equal(open(host).getAttribute("type"), "button");
});
