import assert from "node:assert/strict";
import test, { afterEach, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

const mounted = [];

afterEach(() => {
  while (mounted.length > 0) mounted.pop()();
});

let Practice;
let render;
let document;

before(
  async () => {
    ({ document } = browser(
      "https://tolearn.local/ru/practice/?program=/programs/llm-agents-base&topic=local-runtime",
    ));
    ({ default: Practice } = await island("Practice"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const TOPIC = {
  id: "local-runtime",
  title: "Локальный рантайм",
  practice: {
    kind: "ops",
    tier: "P2",
    task: "Подними модель локально",
    deliverable: "Лог запуска",
    starting_point: null,
    fallback: null,
    time_box_min: 90,
    smoke_checked: false,
    constraints: [],
    acceptance: [],
  },
};

const IDLE = { spent_sec: 0, left_sec: 90 * 60, box_min: 90, running: false, expired: false };

function mount(timer = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "topic") return Promise.resolve(TOPIC);
    if (name === "practice") return Promise.resolve({ ...IDLE, ...timer });
    throw new Error(`лишняя команда ${name}`);
  };
  mounted.push(render(
    () =>
      Practice({
        text: ru,
        locale: "ru",
        program: "/programs/llm-agents-base",
        topic: "local-runtime",
        today: "2026-07-27",
        call,
      }),
    host,
  ));
  return { host, calls };
}

const steps = (calls) =>
  calls.filter(({ name }) => name === "practice").map(({ payload }) => payload.step);

test("таймер показывает остаток от таймбокса, ничего не запуская сам", async () => {
  const { host, calls } = mount();
  await settled();

  assert.equal(host.querySelector("[data-timer]").textContent, "90:00");
  assert.deepEqual(steps(calls), ["peek"]);
  assert.ok(host.querySelector("[data-start]"), host.innerHTML);
  assert.equal(host.querySelector("[data-pause]"), null);
});

test("запуск, пауза и сброс уходят отдельными шагами", async () => {
  const { host, calls } = mount();
  await settled();

  host.querySelector("[data-start]").click();
  await settled();
  host.querySelector("[data-reset]").click();
  await settled();

  assert.deepEqual(steps(calls), ["peek", "start", "reset"]);
});

test("идущий таймер предлагает паузу вместо запуска", async () => {
  const { host, calls } = mount({ running: true, spent_sec: 600, left_sec: 90 * 60 - 600 });
  await settled();

  assert.ok(host.querySelector("[data-pause]"), host.innerHTML);
  assert.equal(host.querySelector("[data-start]"), null);
  host.querySelector("[data-pause]").click();
  await settled();

  assert.deepEqual(steps(calls), ["peek", "pause"]);
});

test("потраченное показано рядом с таймбоксом", async () => {
  const { host } = mount({ spent_sec: 1230, left_sec: 90 * 60 - 1230 });
  await settled();

  const spent = host.querySelector("[data-spent]");
  assert.match(spent.textContent, /20:30/);
  assert.match(spent.textContent, /90/);
});

test("истёкший таймбокс сказан словами и работу не блокирует", async () => {
  const { host } = mount({ running: true, spent_sec: 5700, left_sec: -300, expired: true });
  await settled();

  assert.ok(host.querySelector("[data-expired]"), host.innerHTML);
  assert.equal(host.querySelector("[data-timer]").getAttribute("data-over"), "");
  assert.match(host.querySelector("[data-timer]").textContent, /05:00/);
  assert.ok(host.querySelector("[data-pause]"), "истечение остановило таймер");
  assert.ok(host.querySelector("[data-task]"), "истечение спрятало практику");
});

test("таймер восстанавливается из записи, а не из памяти экрана", async () => {
  const { host, calls } = mount({ running: true, spent_sec: 1800, left_sec: 90 * 60 - 1800 });
  await settled();

  assert.deepEqual(steps(calls), ["peek"]);
  assert.match(host.querySelector("[data-spent]").textContent, /30:00/);
  assert.equal(host.querySelector("[data-timer]").textContent, "60:00");
});
