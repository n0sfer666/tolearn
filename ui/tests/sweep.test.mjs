import assert from "node:assert/strict";
import test, { after, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Sweep;
let render;
let document;

before(async () => {
  ({ document } = browser("https://tolearn.local/ru/sweep/"));
  ({ default: Sweep } = await island("Sweep"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const READY = [
  { topic: "tokens-context-cost", title: "Токены", due: "2026-07-09", overdue: true },
  { topic: "local-runtime", title: "Локальный рантайм", due: "2026-07-20", overdue: true },
  { topic: "structured-output", title: "Структурный вывод", due: "2026-10-24", overdue: false },
];

const SHUT = {
  open: false,
  stale: false,
  done: false,
  stage: "over",
  asked: 0,
  total: 0,
  hint_ready: false,
  ready: READY,
  legs: [],
  log: [],
  seconds: 0,
  tokens: 0,
};

const GOING = {
  ...SHUT,
  open: true,
  stage: "question",
  asked: 1,
  total: 8,
  hint_ready: true,
  legs: [
    { topic: "tokens-context-cost", title: "Токены", asked: 1, total: 4, verdict: null },
    { topic: "local-runtime", title: "Локальный рантайм", asked: 0, total: 4, verdict: null },
  ],
  log: [
    { side: "examiner", text: "Тема «Токены». Откуда взялось 9.8 GB?" },
    { side: "student", text: "Контекст тоже оплачивается." },
  ],
  seconds: 12,
  tokens: 340,
};

const DONE = {
  ...GOING,
  done: true,
  stage: "done",
  legs: [
    { topic: "tokens-context-cost", title: "Токены", asked: 1, total: 4, verdict: "```json{}```" },
    { topic: "local-runtime", title: "Локальный рантайм", asked: 0, total: 4, verdict: null },
  ],
};

const SETTLED = {
  settled: [
    { topic: "tokens-context-cost", title: "Токены", result: "pass", status: "passed" },
  ],
};

const alive = [];

after(() => {
  for (const dispose of alive) dispose();
});

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const states = options.states ?? [SHUT];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "provider") {
      return Promise.resolve({ provider: { enabled: options.enabled ?? true } });
    }
    if (name === "sweep_accept") return Promise.resolve(SETTLED);
    if (name.startsWith("sweep_")) {
      return Promise.resolve(states.length > 1 ? states.shift() : states[0]);
    }
    throw new Error(`лишняя команда ${name}`);
  };
  const dispose = render(
    () =>
      Sweep({
        text: ru,
        locale: "ru",
        program: "/bundle",
        today: "2026-08-06",
        call,
      }),
    host,
  );
  alive.push(dispose);
  return { host, calls, dispose };
}

const ticks = async (times = 4) => {
  for (let i = 0; i < times; i += 1) await new Promise((resolve) => setTimeout(resolve, 4));
};

test("незапущенный прогон показывает пул и метит просроченное", async () => {
  const { host } = mount();
  await ticks();

  const picks = [...host.querySelectorAll("[data-pick]")];
  assert.equal(picks.length, 3);
  assert.equal(picks[0].getAttribute("data-pick"), "tokens-context-cost");
  assert.ok(picks[0].hasAttribute("data-overdue"));
  assert.equal(picks[2].hasAttribute("data-overdue"), false);
  assert.match(picks[0].textContent, new RegExp(ru.sweep.overdue));
});

test("выключенный провайдер не даёт начать прогон", async () => {
  const { host } = mount({ enabled: false });
  await ticks();

  assert.equal(host.querySelector("[data-start]").disabled, true);
  assert.match(host.textContent, new RegExp(ru.sweep.off));
});

test("число тем уходит в запуск", async () => {
  const { host, calls } = mount({ states: [SHUT, GOING] });
  await ticks();

  const field = host.querySelector("[data-topics]");
  field.value = "2";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  host.querySelector("[data-start]").click();
  await ticks();

  const [started] = calls.filter(({ name }) => name === "sweep_start");
  assert.equal(started.payload.topics, 2);
  assert.equal(started.payload.restart, false);
});

test("идущий прогон показывает темы, разговор и общий счёт", async () => {
  const { host } = mount({ states: [GOING] });
  await ticks();

  const legs = [...host.querySelectorAll("[data-leg]")];
  assert.equal(legs.length, 2);
  assert.match(legs[0].querySelector("[data-asked]").textContent, /1 \/ 4/);
  assert.equal(host.querySelectorAll("[data-talk] li").length, 2);
  assert.match(host.querySelector("[data-count]").textContent, /1 \/ 8/);
});

test("ответ уходит одной командой без темы в адресе", async () => {
  const { host, calls } = mount({ states: [GOING] });
  await ticks();

  const field = host.querySelector("[data-answer]");
  field.value = "Контекст тоже оплачивается.";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  host.querySelector("[data-say]").click();
  await ticks();

  const [said] = calls.filter(({ name }) => name === "sweep_say");
  assert.equal(said.payload.text, "Контекст тоже оплачивается.");
  assert.equal(said.payload.today, "2026-08-06");
  assert.equal(host.querySelector("[data-answer]").value, "");
});

test("завершённый прогон просит подтверждения и показывает записанное", async () => {
  const { host, calls } = mount({ states: [DONE] });
  await ticks();

  assert.equal(host.querySelector("[data-reply]"), null);
  host.querySelector("[data-accept]").click();
  await ticks(8);

  assert.equal(calls.filter(({ name }) => name === "sweep_accept").length, 1);
  const written = host.querySelector("[data-settled-topic]");
  assert.match(written.textContent, /pass/);
  assert.match(written.textContent, new RegExp(ru.status.passed));
});

test("устаревшие темы видны и перезапуск идёт с флагом", async () => {
  const { host, calls } = mount({ states: [{ ...GOING, stale: true }] });
  await ticks();

  assert.match(host.querySelector("[data-stale]").textContent, new RegExp(ru.sweep.stale));
  host.querySelector("[data-restart]").click();
  await ticks();

  const [again] = calls.filter(({ name }) => name === "sweep_start");
  assert.equal(again.payload.restart, true);
});
