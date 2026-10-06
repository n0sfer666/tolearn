import assert from "node:assert/strict";
import test, { after, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Progress;
let Steps;
let render;
let document;

before(async () => {
  ({ document } = browser("https://tolearn.local/ru/new/"));
  ({ default: Progress } = await island("Progress", "src/components/generate"));
  ({ default: Steps } = await island("Steps", "src/components/generate"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const alive = [];

after(() => {
  for (const dispose of alive) dispose();
});

const working = (options = {}) => ({
  step: () => null,
  running: () => options.running ?? true,
  cancelling: () => false,
  ended: () => null,
  calm: () => false,
  refused: () => null,
  refuse: () => {},
  run: () => Promise.resolve(null),
  cancel: () => {},
  said: () => options.said ?? ru.generate.working,
  spent: () => options.spent ?? 0,
});

function mount(component, props) {
  const host = document.createElement("div");
  document.body.append(host);
  alive.push(render(() => component(props), host));
  return host;
}

const SEEN = [
  { key: "asked", label: ru.generate.markAsked, done: true, spent: null },
  { key: "reach", label: ru.generate.markReach, done: true, spent: 1_200 },
  { key: "drawn", label: ru.generate.markDrawn, done: false, spent: null },
];

test("ожидание показывает живой индикатор и счётчик с первой секунды", () => {
  const host = mount(Progress, { text: ru, work: working() });

  const spin = host.querySelector("[data-progress] [data-spin]");
  assert.ok(spin !== null, "живого индикатора нет");
  assert.equal(spin.getAttribute("aria-hidden"), "true", "индикатор читается голосом");
  assert.equal(host.querySelector("[data-progress] [data-elapsed]").textContent, "0 с");
  assert.equal(host.querySelector("[data-progress] [data-step]").textContent, ru.generate.working);
  assert.ok(host.querySelector("[data-patience]") === null, "подсказка пришла до минуты");
});

test("до минуты подсказки терпения нет, с минуты она появляется", () => {
  const early = mount(Progress, { text: ru, work: working({ spent: 59_000 }) });
  assert.equal(early.querySelector("[data-progress] [data-elapsed]").textContent, "59 с");
  assert.ok(early.querySelector("[data-patience]") === null, "подсказка пришла раньше минуты");

  const late = mount(Progress, { text: ru, work: working({ spent: 80_000 }) });
  assert.equal(late.querySelector("[data-progress] [data-elapsed]").textContent, "1:20");
  assert.equal(late.querySelector("[data-patience]").textContent, ru.generate.patience);
});

test("в ленте текущая отметка стоит «в работе»: свой индикатор, шаг в настоящем времени и счётчик", () => {
  const said = ru.generate.stepPlan;
  const host = mount(Steps, { text: ru, seen: SEEN, work: working({ said, spent: 80_000 }) });

  const busy = host.querySelector("[data-mark][data-busy]");
  assert.equal(busy.dataset.mark, "drawn", "в работе отмечена не текущая отметка");
  assert.equal(busy.querySelector("[data-label]").textContent, said, "текущий шаг назван не в настоящем времени");
  assert.equal(busy.querySelector("[data-elapsed]").textContent, "1:20");
  assert.ok(busy.querySelector("[data-spin]") !== null, "у текущей отметки нет индикатора");
  assert.ok(busy.querySelector("[data-at]") === null, "у незавершённой отметки взялось итоговое время");

  const done = host.querySelector('[data-mark="reach"]');
  assert.equal(done.querySelector("[data-at]").textContent, "1 с");
  assert.equal(done.querySelector("[data-label]").textContent, ru.generate.markReach);
  assert.ok(done.querySelector("[data-spin]") === null, "у пройденной отметки крутится индикатор");
});

test("остановленная генерация оставляет ленту без «в работе»", async () => {
  const host = mount(Steps, { text: ru, seen: SEEN, work: working({ running: false }) });
  await settled();

  assert.ok(host.querySelector("[data-mark][data-busy]") === null, "лента крутится при остановленной работе");
  assert.equal(host.querySelector('[data-mark="drawn"] [data-label]').textContent, ru.generate.markDrawn);
  assert.ok(host.querySelector("[data-elapsed]") === null, "счётчик идёт без работы");
});
