import assert from "node:assert/strict";
import test, { after, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";
import { rule } from "./support/styles.mjs";

const SVG = "data:image/svg+xml;base64,PHN2Zy8+";

let Picture;
let render;
let window;
let document;

before(
  async () => {
    window = browser("https://tolearn.local/ru/stage/?program=chip&node=rom&stage=voices");
    document = window.document;
    ({ default: Picture } = await island("Picture", "src/components/reading"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const alive = [];

after(() => {
  for (const dispose of alive) dispose();
});

const drawn = (kind) => ({
  id: "d1",
  kind,
  text: "Схема каналов",
  lang: null,
  src: SVG,
  license: null,
  attribution: null,
  source: null,
});

function mount(kind = "diagram") {
  const host = document.createElement("div");
  document.body.append(host);
  alive.push(render(() => Picture({ block: drawn(kind), text: ru }), host));
  return host;
}

const opener = (host) => host.querySelector("[data-zoom-open]");
const layer = () => document.querySelector("[data-zoom]");
const back = () => document.querySelector("[data-zoom-back]");
const shut = () => document.querySelector("[data-zoom-close]");
const frame = () => document.querySelector("[data-zoom-scroll]");

function key(name, node, shifted = false) {
  const event = new window.Event("keydown", { bubbles: true, cancelable: true });
  Object.defineProperty(event, "key", { value: name });
  Object.defineProperty(event, "shiftKey", { value: shifted });
  (node ?? document.body).dispatchEvent(event);
}

test("картинка в тексте не выше доли экрана и держит пропорции", () => {
  const body = rule("reading.css", "figure[data-block] img");

  assert.match(body, /max-inline-size: 100%/);
  assert.match(body, /max-block-size: 60vh/, "высота картинки в тексте не ограничена");
  assert.match(body, /object-fit: contain/, "ужатая картинка исказится");
});

test("клик по картинке разворачивает её слоем поверх этапа", async () => {
  const host = mount();
  await settled();

  assert.equal(layer(), null);
  opener(host).click();
  await settled();

  assert.equal(layer().getAttribute("role"), "dialog");
  assert.equal(layer().getAttribute("aria-modal"), "true");
  assert.equal(frame().querySelector("img").getAttribute("src"), SVG);
  assert.equal(frame().querySelector("img").getAttribute("alt"), ru.stage.diagram);

  shut().click();
  await settled();
  assert.equal(layer(), null);
});

test("в слое картинка идёт натуральным размером со скроллом по обеим осям", () => {
  const box = rule("zoom.css", "[data-zoom-scroll]");
  const drawing = rule("zoom.css", "[data-zoom-scroll] img");

  assert.match(box, /overflow: auto/, "большую картинку в слое не прокрутить");
  assert.match(
    box,
    /max-block-size: \d+vh/,
    "высота слоя мерится от родителя с авто-высотой — процент не разрешается и прокрутки нет",
  );
  assert.match(drawing, /max-inline-size: none/, "слой ужимает картинку так же, как текст");
  assert.match(drawing, /max-block-size: none/);

  const shade = rule("zoom.css", "[data-zoom-back]");
  assert.match(shade, /position: fixed/);
  assert.match(shade, /z-index: var\(--z-overlay\)/);
});

test("Esc закрывает слой и возвращает фокус на картинку", async () => {
  const host = mount();
  await settled();

  const heard = [];
  const listen = () => heard.push("escape");
  document.addEventListener("keydown", listen);

  opener(host).click();
  await settled();
  assert.notEqual(layer(), null);

  key("Escape", layer());
  await settled();

  assert.equal(layer(), null);
  assert.deepEqual(heard, [], "Esc из слоя дошёл до экрана и увёл со страницы");
  assert.equal(document.activeElement, opener(host), "фокус не вернулся на картинку");

  document.removeEventListener("keydown", listen);
});

test("клик по затемнению закрывает слой, клик по самой картинке — нет", async () => {
  const host = mount();
  await settled();

  opener(host).click();
  await settled();

  frame().click();
  await settled();
  assert.notEqual(layer(), null, "слой закрылся от клика по картинке");

  back().click();
  await settled();
  assert.equal(layer(), null);
});

test("Tab ходит по слою и не выпадает на текст под ним", async () => {
  const host = mount();
  await settled();

  opener(host).click();
  await settled();

  assert.equal(document.activeElement, shut(), "фокус не встал в слой");

  key("Tab", shut());
  await settled();
  assert.equal(document.activeElement, frame());

  key("Tab", frame());
  await settled();
  assert.equal(document.activeElement, shut());

  key("Tab", shut(), true);
  await settled();
  assert.equal(document.activeElement, frame(), "Shift+Tab не идёт назад по слою");

  shut().click();
  await settled();
});

test("кнопка разворота подписана и не ломает подпись картинки", async () => {
  const host = mount("image");
  await settled();

  assert.equal(opener(host).getAttribute("type"), "button");
  assert.equal(opener(host).getAttribute("aria-label"), `${ru.stage.expand}: Схема каналов`);
  assert.equal(opener(host).querySelector("img").getAttribute("alt"), "Схема каналов");
});
