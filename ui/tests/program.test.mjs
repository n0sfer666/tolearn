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

const span = (min, max) => ({ min, max });

const TALLY = {
  done: 1,
  total: 4,
  stale: 0,
  share: 0.25,
  hours_done: span(4, 6),
  hours_total: span(14, 20),
};

const OUT = {
  program: TALLY,
  stages: [
    { n: 1, title: "Модели и доступ", checkpoint: "cp-gateway", tally: TALLY },
    { n: 2, title: "Агенты и инструменты", checkpoint: "cp-agent-stack", tally: TALLY },
  ],
  topics: [
    {
      id: "local-runtime",
      title: "Локальный рантайм",
      stage: 1,
      checkpoint: false,
      status: "passed",
      hours: span(4, 6),
      blocked_by: [],
    },
    {
      id: "cp-gateway",
      title: "Шлюз к моделям",
      stage: 1,
      checkpoint: true,
      status: "blocked",
      hours: span(3, 5),
      blocked_by: [{ id: "openai-compatible-api", title: "Совместимый API" }],
    },
    {
      id: "agent-loop",
      title: "Цикл агента",
      stage: 2,
      checkpoint: false,
      status: "todo",
      hours: span(4, 5),
      blocked_by: [],
    },
  ],
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "program") return Promise.resolve(options.out ?? OUT);
    throw new Error(`лишняя команда ${name}`);
  };
  render(
    () =>
      Program({
        text: ru,
        locale: "ru",
        path: options.path ?? "/programs/llm-agents-base",
        today: "2026-07-27",
        call,
      }),
    host,
  );
  return { host, calls };
}

const at = (host, id) => host.querySelector(`[data-topic="${id}"]`);

test("экран читает программу по её пути", async () => {
  const { calls } = mount({ path: "/programs/other" });
  await settled();

  assert.deepEqual(calls, [
    { name: "program", payload: { bundle: "/programs/other", today: "2026-07-27" } },
  ]);
});

test("без явного пути программа берётся из адреса страницы", async () => {
  const { calls } = mount({ path: null });
  await settled();

  assert.equal(calls[0].payload.bundle, "/programs/llm-agents-base");
});

test("этапы идут по порядку, тема стоит в своём этапе", async () => {
  const { host } = mount();
  await settled();

  const stages = [...host.querySelectorAll("[data-stage]")];
  assert.deepEqual(
    stages.map((stage) => stage.dataset.stage),
    ["1", "2"],
  );
  assert.ok(stages[0].querySelector('[data-topic="local-runtime"]'), "тема не в своём этапе");
  assert.ok(stages[1].querySelector('[data-topic="agent-loop"]'), "тема не в своём этапе");
  assert.equal(stages[0].querySelector('[data-topic="agent-loop"]'), null, "тема попала в чужой этап");
});

test("статус читается без цвета: глиф с подписью и текст рядом", async () => {
  const { host } = mount();
  await settled();

  const passed = at(host, "local-runtime");
  const glyph = passed.querySelector("[role=img]");
  assert.equal(glyph.getAttribute("aria-label"), ru.status.passed);
  assert.match(passed.textContent, new RegExp(ru.status.passed), passed.textContent);
  assert.notEqual(
    at(host, "agent-loop").querySelector("[role=img]").textContent,
    glyph.textContent,
    "разные статусы получили один глиф",
  );
});

test("чекпойнт отличается от обычной темы", async () => {
  const { host } = mount();
  await settled();

  const gate = at(host, "cp-gateway");
  assert.equal(gate.dataset.checkpoint, "");
  assert.match(gate.textContent, new RegExp(ru.program.checkpoint));
  assert.equal(at(host, "local-runtime").dataset.checkpoint, undefined);
});

test("заблокированная тема говорит, чем разблокируется, и не кликается", async () => {
  const { host } = mount();
  await settled();

  const gate = at(host, "cp-gateway");
  assert.match(gate.textContent, /Совместимый API/, gate.textContent);
  assert.match(gate.textContent, new RegExp(ru.program.blockedBy));
  assert.equal(gate.querySelector("a"), null, "заблокированная тема осталась ссылкой");
});

test("доступная тема ведёт на свой экран", async () => {
  const { host } = mount();
  await settled();

  const link = at(host, "local-runtime").querySelector("a");
  assert.equal(
    link.getAttribute("href"),
    "/ru/topic/?program=%2Fprograms%2Fllm-agents-base&topic=local-runtime",
  );
});

test("этап показывает свой счётчик, а не общий на всех", async () => {
  const { host } = mount({
    out: {
      ...OUT,
      stages: [
        { ...OUT.stages[0], tally: { ...TALLY, done: 1, total: 2 } },
        { ...OUT.stages[1], tally: { ...TALLY, done: 0, total: 2 } },
      ],
    },
  });
  await settled();

  const counts = [...host.querySelectorAll("[data-stage] [data-tally]")].map(
    (node) => node.textContent,
  );
  assert.match(counts[0], /1\D+2/, counts[0]);
  assert.match(counts[1], /0\D+2/, counts[1]);
});

test("счётчик тем стоит в форме множественного числа своего языка", async () => {
  const { host } = mount({
    out: { ...OUT, stages: [{ ...OUT.stages[0], tally: { ...TALLY, done: 1, total: 8 } }] },
  });
  await settled();

  assert.match(host.querySelector("[data-tally]").textContent, /8 тем$/);
});

test("часы темы показаны интервалом", async () => {
  const { host } = mount();
  await settled();

  assert.match(at(host, "local-runtime").textContent, /4\D+6/);
});

test("фильтр оставляет в этапах только совпавшие темы", async () => {
  const { host } = mount();
  await settled();

  const field = host.querySelector("[data-filter]");
  field.value = "шлюз";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();

  assert.equal(host.querySelector('[data-topic="local-runtime"]'), null);
  assert.ok(host.querySelector('[data-topic="cp-gateway"]'), "совпавшая тема пропала");
});

test("из программы есть ход в её дайджест устаревания", async () => {
  const { host } = mount({ path: "/programs/other" });
  await settled();

  const link = host.querySelector("[data-stale]");
  assert.ok(link, host.innerHTML);
  assert.equal(link.getAttribute("href"), "/ru/stale/?program=%2Fprograms%2Fother");
});
