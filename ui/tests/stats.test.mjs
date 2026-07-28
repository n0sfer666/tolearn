import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Stats;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/stats/?program=/programs/llm-agents-base"));
    ({ default: Stats } = await island("Stats"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const OUT = {
  attempts: 7,
  enough: true,
  hinted: 3,
  hinted_share: 3 / 7,
  kinds: [
    { kind: "diagnose", ok: 1, partial: 1, miss: 1 },
    { kind: "tradeoff", ok: 0, partial: 1, miss: 2 },
  ],
  actions: [
    { action: "retry_failed", count: 4 },
    { action: "proceed", count: 1 },
  ],
  streak: { longest: 3, topic: "structured-output" },
  calibration: ["Уверенность выше уровня: отвечал без оговорок."],
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const call = (name) => {
    if (name === "stats") return Promise.resolve({ ...OUT, ...options.out });
    throw new Error(`лишняя команда ${name}`);
  };
  const dispose = render(
    () => Stats({ text: ru, locale: "ru", path: "/programs/llm-agents-base", call }),
    host,
  );
  return { host, dispose };
}

test("доля подсказок показана дробью, а не одним процентом", async () => {
  const { host } = mount();
  await settled();

  const hinted = host.querySelector("[data-hinted]");
  assert.ok(hinted, host.innerHTML);
  assert.match(hinted.textContent, /3/);
  assert.match(hinted.textContent, /7/);
  assert.match(hinted.textContent, /43\s*%/);
});

test("ответы разложены по типам вопросов", async () => {
  const { host } = mount();
  await settled();

  const row = host.querySelector("[data-kind='diagnose']");
  assert.ok(row, host.innerHTML);
  assert.equal(row.querySelector("[data-ok]").textContent, "1");
  assert.equal(row.querySelector("[data-partial]").textContent, "1");
  assert.equal(row.querySelector("[data-miss]").textContent, "1");
});

test("следующие шаги показаны с частотой", async () => {
  const { host } = mount();
  await settled();

  const action = host.querySelector("[data-action='retry_failed']");
  assert.ok(action, host.innerHTML);
  assert.match(action.textContent, /4/);
});

test("серия провалов названа вместе с темой", async () => {
  const { host } = mount();
  await settled();

  const streak = host.querySelector("[data-streak]");
  assert.ok(streak, host.innerHTML);
  assert.match(streak.textContent, /3/);
  assert.match(streak.textContent, /structured-output/);
});

test("на малой выборке вместо выводов сказано, что данных мало", async () => {
  const { host } = mount({
    out: { attempts: 4, enough: false, hinted: 0, hinted_share: 0, kinds: [], actions: [] },
  });
  await settled();

  assert.ok(host.querySelector("[data-scarce]"), host.innerHTML);
  assert.equal(host.querySelector("[data-kinds]"), null, host.innerHTML);
  assert.equal(host.querySelector("[data-hinted]"), null, host.innerHTML);
  assert.match(host.querySelector("[data-attempts]").textContent, /4/);
});

test("самооценка показана как есть и в счёт не идёт", async () => {
  const { host } = mount({ out: { calibration: ["Оценка совпала с результатом."] } });
  await settled();

  const block = host.querySelector("[data-calibration]");
  assert.ok(block, host.innerHTML);
  assert.match(block.textContent, /Оценка совпала с результатом\./);
});

test("самооценка остаётся видна и на малой выборке", async () => {
  const { host } = mount({
    out: { attempts: 2, enough: false, kinds: [], actions: [], calibration: ["Переоценил себя."] },
  });
  await settled();

  assert.ok(host.querySelector("[data-calibration]"), host.innerHTML);
  assert.ok(host.querySelector("[data-scarce]"), host.innerHTML);
});
