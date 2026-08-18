import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Program;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/program/?program=/programs/llm-agents-base"));
    ({ default: Program } = await island("Program"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const TALLY = {
  done: 1,
  total: 2,
  stale: 0,
  share: 0.5,
  hours_done: { min: 4, max: 6 },
  hours_total: { min: 8, max: 12 },
};

const OUT = {
  program: TALLY,
  stages: [{ n: 1, title: "Модели и доступ", checkpoint: null, tally: TALLY }],
  topics: [
    {
      id: "local-runtime",
      title: "Локальный рантайм",
      stage: 1,
      checkpoint: false,
      status: "todo",
      hours: { min: 4, max: 6 },
      blocked_by: [],
    },
  ],
};

const COST = {
  materials: 12,
  held: 4,
  used: 200 * 1024 * 1024,
  budget: 512 * 1024 * 1024,
  spare: 30 * 1024 * 1024,
  need: 60 * 1024 * 1024,
  tight: false,
};

const STATE = {
  total: 12,
  done: 3,
  current: "https://example.invalid/quant",
  title: "Гайд по квантованию",
  topic: "Локальный рантайм",
  topic_at: 2,
  topics: 5,
  finished: false,
  cancelled: false,
  bytes: 0,
  saved: [],
  skipped: [],
  failed: [],
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  let step = 0;
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "program") return Promise.resolve(OUT);
    if (name === "offline_cost") return Promise.resolve({ ...COST, ...(options.cost ?? {}) });
    if (name === "save_offline") return Promise.resolve({ job: "job-7", total: 12 });
    if (name === "offline_state") {
      const states = options.states ?? [STATE];
      const out = states[Math.min(step, states.length - 1)];
      step += 1;
      return Promise.resolve(out);
    }
    if (name === "stop_offline") return Promise.resolve({ stopping: true });
    throw new Error(`лишняя команда ${name}`);
  };
  const dispose = render(
    () =>
      Program({
        text: ru,
        locale: "ru",
        path: "/programs/llm-agents-base",
        today: "2026-07-27",
        call,
        save: () => Promise.resolve(null),
      }),
    host,
  );
  return { host, calls, dispose };
}

const tick = async (times) => {
  for (let index = 0; index < times; index += 1) {
    await new Promise((resolve) => setTimeout(resolve, 350));
  }
};

const button = (host) => host.querySelector("[data-unload-whole]");

test("умещается в бюджет — выгрузка всей программы идёт без вопросов", async () => {
  const { host, calls, dispose } = mount();
  await settled();

  assert.equal(button(host).textContent, ru.offline.whole);
  button(host).click();
  await settled();

  assert.deepEqual(
    calls.find((made) => made.name === "offline_cost"),
    { name: "offline_cost", payload: { bundle: "/programs/llm-agents-base" } },
  );
  assert.deepEqual(
    calls.find((made) => made.name === "save_offline").payload,
    { bundle: "/programs/llm-agents-base", topic: null, again: null },
    "программная выгрузка ушла не за все темы",
  );
  assert.equal(host.querySelector("[data-unload-price]"), null, "спросили там, где места хватает");

  dispose();
});

test("места впритык — цена названа до старта и старт ждёт согласия", async () => {
  const { host, calls, dispose } = mount({ cost: { tight: true } });
  await settled();

  button(host).click();
  await settled();

  const price = host.querySelector("[data-unload-price]");
  assert.match(price.querySelector("[data-unload-tight]").textContent, new RegExp(ru.offline.tight));
  assert.match(price.querySelector("[data-price-pieces]").textContent, /12/);
  assert.match(price.querySelector("[data-price-kept]").textContent, /4/);
  assert.match(price.querySelector("[data-price-used]").textContent, /200 \/ 512/);
  assert.match(price.querySelector("[data-price-need]").textContent, /60/);
  assert.equal(
    calls.some((made) => made.name === "save_offline"),
    false,
    "выгрузка стартовала до согласия",
  );

  price.querySelector("[data-unload-go]").click();
  await settled();

  assert.equal(calls.filter((made) => made.name === "save_offline").length, 1);
  assert.equal(host.querySelector("[data-unload-price]"), null);

  dispose();
});

test("отказ на предупреждении не начинает выгрузку", async () => {
  const { host, calls, dispose } = mount({ cost: { tight: true } });
  await settled();

  button(host).click();
  await settled();
  host.querySelector("[data-unload-back]").click();
  await settled();

  assert.equal(host.querySelector("[data-unload-price]"), null);
  assert.equal(calls.some((made) => made.name === "save_offline"), false);
  assert.equal(button(host).disabled, false, "кнопка осталась запертой после отказа");

  dispose();
});

test("прогресс называет тему и материал, а выгрузку можно остановить", async () => {
  const { host, calls, dispose } = mount();
  await settled();

  button(host).click();
  await settled();

  const line = host.querySelector("[data-whole-progress]").textContent;
  assert.match(line, /тема 2 из 5/);
  assert.match(line, /Локальный рантайм/);
  assert.match(line, /материал 4 из 12/);
  assert.match(line, /Гайд по квантованию/);

  host.querySelector("[data-whole-stop]").click();
  await settled();

  assert.deepEqual(calls.at(-1), { name: "stop_offline", payload: { job: "job-7" } });

  dispose();
});

test("отчёт называет павших поимённо, а повтор берёт ровно их", async () => {
  const broken = {
    ...STATE,
    done: 12,
    current: "",
    title: "",
    finished: true,
    bytes: 4 * 1024 * 1024,
    saved: ["https://example.invalid/quant"],
    failed: [
      { url: "https://example.invalid/repo", title: "Репозиторий агента", why: "сеть не ответила" },
    ],
  };
  const { host, calls, dispose } = mount({ states: [broken] });
  await settled();

  button(host).click();
  await tick(1);

  assert.match(host.querySelector("[data-whole-report]").textContent, /1 сохранено/);
  assert.deepEqual(
    [...host.querySelectorAll("[data-whole-failed] li")].map((row) => row.textContent),
    ["Репозиторий агента — сеть не ответила"],
  );

  const again = button(host);
  assert.equal(again.textContent, ru.offline.again);
  again.click();
  await settled();

  assert.equal(
    calls.filter((made) => made.name === "offline_cost").length,
    1,
    "повтор упавших снова спросил цену",
  );
  assert.deepEqual(calls.filter((made) => made.name === "save_offline").at(-1).payload, {
    bundle: "/programs/llm-agents-base",
    topic: null,
    again: "job-7",
  });

  dispose();
});
