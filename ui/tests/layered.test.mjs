import assert from "node:assert/strict";
import test, { before, beforeEach } from "node:test";

import { island } from "../scripts/island.mjs";
import { browser, settled } from "./support/dom.mjs";

let layered;
let window;

before(
  async () => {
    window = browser();
    ({ layered } = await island("layered", "src/lib", "ts"));
  },
  { timeout: 300_000 },
);

beforeEach(() => {
  window.document.body.innerHTML = "";
});

function build(picks) {
  const outside = window.document.createElement("button");
  const back = window.document.createElement("div");
  const box = window.document.createElement("div");
  box.innerHTML =
    '<button data-first></button><div data-scroll tabindex="0"></div><button data-last></button>';
  back.append(box);
  window.document.body.append(outside, back);
  const shut = [];
  const layer = layered(
    () => box,
    () => shut.push(true),
    picks,
  );
  box.addEventListener("keydown", layer.keyed);
  box.addEventListener("focusout", layer.held);
  back.addEventListener("click", layer.aside);
  return { box, back, outside, shut };
}

const at = (box, selector) => box.querySelector(selector);

function press(node, key, init = {}) {
  const event = new window.KeyboardEvent("keydown", {
    key,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  node.dispatchEvent(event);
  return event;
}

function clicked(node) {
  node.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }));
}

test("Esc закрывает слой и не доходит до экрана", () => {
  const { box, shut } = build();
  const seen = [];
  window.document.addEventListener("keydown", () => seen.push(true));

  const event = press(at(box, "[data-first]"), "Escape");

  assert.equal(shut.length, 1);
  assert.equal(event.defaultPrevented, true);
  assert.deepEqual(seen, []);
});

test("Tab и Shift+Tab ходят по кругу внутри слоя", () => {
  const { box } = build();
  const first = at(box, "[data-first]");
  const last = at(box, "[data-last]");
  first.focus();

  press(first, "Tab");
  assert.equal(window.document.activeElement, last);
  press(last, "Tab");
  assert.equal(window.document.activeElement, first);
  press(first, "Tab", { shiftKey: true });
  assert.equal(window.document.activeElement, last);
});

test("свой набор стопов ставит в кольцо и прокручиваемую рамку", () => {
  const { box } = build("button, [data-scroll]");
  const first = at(box, "[data-first]");
  first.focus();

  press(first, "Tab");

  assert.equal(window.document.activeElement, at(box, "[data-scroll]"));
});

test("фокус, ушедший из слоя, возвращается на первый стоп", async () => {
  const { box, outside } = build();

  at(box, "[data-last]").focus();
  outside.focus();
  await settled();

  assert.equal(window.document.activeElement, at(box, "[data-first]"));
});

test("переход фокуса внутри слоя ничего не перехватывает", async () => {
  const { box } = build();
  const last = at(box, "[data-last]");

  at(box, "[data-first]").focus();
  last.focus();
  await settled();

  assert.equal(window.document.activeElement, last);
});

test("клик по фону закрывает слой, клик по самому слою — нет", () => {
  const { box, back, shut } = build();

  clicked(at(box, "[data-first]"));
  assert.deepEqual(shut, []);

  clicked(back);
  assert.equal(shut.length, 1);
});

test("прочие клавиши слой не перехватывает", () => {
  const { box, shut } = build();

  const event = press(at(box, "[data-first]"), "ArrowDown");

  assert.deepEqual(shut, []);
  assert.equal(event.defaultPrevented, false);
});
