import assert from "node:assert/strict";
import test, { after, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Talk;
let render;
let document;
let window;

before(async () => {
  window = browser("https://tolearn.local/ru/exam/dialog/");
  ({ document } = window);
  ({ default: Talk } = await island("Talk", "src/components/exam"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const alive = [];

after(() => {
  for (const dispose of alive) dispose();
});

function mount(text, copy) {
  const host = document.createElement("div");
  document.body.append(host);
  const dispose = render(
    () => Talk({ text: ru.dialog, log: [{ side: "examiner", text }], copy }),
    host,
  );
  alive.push(dispose);
  return host;
}

test("реплика раскладывается на блоки, а не идёт простынёй", async () => {
  const host = mount("Порядок такой. 1. Запиши предсказание. 2. Прогони `ollama ps`.");
  await settled();

  assert.equal(host.querySelectorAll("li p").length, 1);
  assert.equal(host.querySelectorAll("ol[data-rich] li").length, 2);
  assert.match(host.querySelector("ol[data-rich] li").textContent, /Запиши предсказание/);
});

test("огороженный блок кода рисуется отдельной областью", async () => {
  const host = mount("Запусти:\n\n```sh\nollama ps\n```");
  await settled();

  assert.equal(host.querySelector("pre [data-snip]").textContent, "ollama ps");
});

test("клик по коду кладёт его в буфер и говорит об этом", async () => {
  const copied = [];
  const host = mount("сохрани вывод `ollama ps` в файл", (text) => {
    copied.push(text);
    return Promise.resolve();
  });
  const said = toasts(window);
  await settled();

  host.querySelector("[data-snip]").click();
  await settled();

  assert.deepEqual(copied, ["ollama ps"]);
  assert.deepEqual(said.at(-1), { tone: "ok", text: ru.dialog.copied });
});

test("недоступный буфер честно просит скопировать руками", async () => {
  const host = mount("сохрани вывод `ollama ps` в файл", () => Promise.reject(new Error("no")));
  const said = toasts(window);
  await settled();

  host.querySelector("[data-snip]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "warn", text: ru.dialog.copyManually });
});

test("код в реплике доступен с клавиатуры и подписан", async () => {
  const host = mount("вывод `ollama ps` в файл");
  await settled();

  const snip = host.querySelector("[data-snip]");

  assert.equal(snip.tagName, "BUTTON");
  assert.equal(snip.getAttribute("type"), "button");
  assert.equal(snip.getAttribute("aria-label"), `${ru.dialog.copyCode}: ollama ps`);
});
