import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Plan;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/program/?program=/programs/llm-agents-base"));
    ({ default: Plan } = await island("Plan"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const OUT = {
  weekly_hours: 6,
  daily_hours: 6 / 7,
  left: { min: 14, max: 20 },
  unknown: 5,
  soonest: { days: 17, date: "2026-08-14" },
  latest: { days: 24, date: "2026-08-21" },
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "plan") return Promise.resolve({ ...OUT, ...options.out });
    throw new Error(`лишняя команда ${name}`);
  };
  const dispose = render(
    () => Plan({ text: ru, locale: "ru", path: "/programs/llm-agents-base", today: "2026-07-28", call }),
    host,
  );
  return { host, calls, dispose };
}

test("дневная норма показана вместе с недельным бюджетом", async () => {
  const { host } = mount();
  await settled();

  const norm = host.querySelector("[data-norm]");
  assert.ok(norm, host.innerHTML);
  assert.match(norm.textContent, /6/);
  assert.match(norm.textContent, /0[.,]9/, norm.textContent);
});

test("прогноз показан формулой, а не готовым числом", async () => {
  const { host } = mount();
  await settled();

  const formula = host.querySelector("[data-formula]");
  assert.ok(formula, host.innerHTML);
  assert.match(formula.textContent, /14/);
  assert.match(formula.textContent, /20/);
  assert.match(formula.textContent, /17/);
  assert.match(formula.textContent, /24/);
});

test("дата завершения — интервал из двух дат", async () => {
  const { host } = mount();
  await settled();

  const finish = host.querySelector("[data-finish]");
  assert.ok(finish, host.innerHTML);
  assert.match(finish.textContent, /2026-08-14/);
  assert.match(finish.textContent, /2026-08-21/);
});

test("несгенерированные темы названы неизвестным остатком", async () => {
  const { host } = mount();
  await settled();

  const unknown = host.querySelector("[data-unknown]");
  assert.ok(unknown, host.innerHTML);
  assert.match(unknown.textContent, /5/);
  assert.match(unknown.textContent, new RegExp(ru.plan.unknownLead));
});

test("когда написаны все темы, про неизвестный остаток не врут", async () => {
  const { host } = mount({ out: { unknown: 0 } });
  await settled();

  assert.equal(host.querySelector("[data-unknown]"), null, host.innerHTML);
});

test("пройденная программа не обещает работы на будущее", async () => {
  const { host } = mount({
    out: {
      left: { min: 0, max: 0 },
      soonest: { days: 0, date: "2026-07-28" },
      latest: { days: 0, date: "2026-07-28" },
      unknown: 0,
    },
  });
  await settled();

  assert.match(host.textContent, new RegExp(ru.plan.done));
  assert.equal(host.querySelector("[data-formula]"), null, host.innerHTML);
});
