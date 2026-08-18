import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Queue;
let render;
let document;

before(async () => {
  ({ document } = browser());
  ({ default: Queue } = await island("Queue"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const DUE = [
  {
    program: "llm-agents-base",
    title: "Работа с агентами LLM",
    bundle: "/programs/llm-agents-base",
    topic: "tokens-context-cost",
    topic_title: "Токены, контекст и стоимость",
    due: "2026-07-09",
    overdue: true,
  },
  {
    program: "llm-agents-extra",
    title: "Продолжение про агентов",
    bundle: "/programs/llm-agents-extra",
    topic: "openai-compatible-api",
    topic_title: "OpenAI-совместимый контракт",
    due: "2026-07-28",
    overdue: false,
  },
];

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const answers = [options.due ?? DUE, options.after ?? []];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "queue") return Promise.resolve({ due: answers.shift() ?? [] });
    if (name === "repeat") {
      if (options.refuses) return Promise.reject(options.refuses);
      return Promise.resolve({ next_review_at: "2026-10-26" });
    }
    throw new Error(`лишняя команда ${name}`);
  };
  const said = toasts(document.defaultView);
  const dispose = render(
    () => Queue({ text: ru, locale: "ru", today: "2026-07-28", call }),
    host,
  );
  return { host, calls, dispose, said };
}

const only = (calls, name) => calls.filter((call) => call.name === name);

test("очередь показывает просроченное отдельно от сегодняшнего", async () => {
  const { host } = mount();
  await settled();

  const overdue = host.querySelector("[data-overdue]");
  const today = host.querySelector("[data-today]");
  assert.ok(overdue, `нет группы просроченного: ${host.innerHTML}`);
  assert.ok(today, `нет группы на сегодня: ${host.innerHTML}`);
  assert.match(overdue.textContent, /Токены, контекст и стоимость/);
  assert.match(today.textContent, /OpenAI-совместимый контракт/);
  assert.doesNotMatch(overdue.textContent, /OpenAI-совместимый контракт/);
});

test("тема очереди названа вместе со своей программой и датой", async () => {
  const { host } = mount();
  await settled();

  const first = host.querySelector("[data-due='tokens-context-cost']");
  assert.ok(first, host.innerHTML);
  assert.match(first.textContent, /Работа с агентами LLM/);
  assert.match(first.textContent, /2026-07-09/);
});

test("из очереди можно уйти прямо в тему", async () => {
  const { host } = mount();
  await settled();

  const link = host.querySelector("[data-due='tokens-context-cost'] a");
  assert.ok(link, host.innerHTML);
  assert.equal(
    link.getAttribute("href"),
    "/ru/topic/?program=%2Fprograms%2Fllm-agents-base&topic=tokens-context-cost",
  );
});

test("повторение уносит тему из очереди и не трогает статус", async () => {
  const { host, calls } = mount();
  await settled();

  host.querySelector("[data-due='tokens-context-cost'] [data-repeat]").dispatchEvent(
    new window.Event("click", { bubbles: true }),
  );
  await settled();
  await settled();

  const repeated = only(calls, "repeat");
  assert.equal(repeated.length, 1);
  assert.deepEqual(repeated[0].payload, {
    bundle: "/programs/llm-agents-base",
    topic: "tokens-context-cost",
    today: "2026-07-28",
  });
  assert.equal(only(calls, "queue").length, 2, "очередь не перечитана");
  assert.ok(
    !calls.some((call) => call.name === "set_status" || call.name === "apply_verdict"),
    "повторение полезло в статус",
  );
  assert.doesNotMatch(host.textContent, /Токены, контекст и стоимость/);
});

test("пустая очередь так и говорит", async () => {
  const { host } = mount({ due: [] });
  await settled();

  assert.match(host.textContent, new RegExp(ru.queue.empty));
  assert.equal(host.querySelector("[data-overdue]"), null);
});

test("отказ в повторении объясняется и тему не теряет", async () => {
  const { host, said } = mount({
    refuses: { code: "unwritable", message: "папка только для чтения" },
  });
  await settled();

  host.querySelector("[data-due='tokens-context-cost'] [data-repeat]").dispatchEvent(
    new window.Event("click", { bubbles: true }),
  );
  await settled();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: "папка только для чтения" });
  assert.match(host.textContent, /Токены, контекст и стоимость/);
});
