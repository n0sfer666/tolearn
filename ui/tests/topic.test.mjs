import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Topic;
let render;
let document;

before(
  async () => {
    ({ document } = browser(
      "https://tolearn.local/ru/topic/?program=/programs/llm-agents-base&topic=local-runtime",
    ));
    ({ default: Topic } = await island("Topic"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const CHECK = (id) => ({ id, claim: `утверждение ${id}`, check: `команда ${id}`, expect: `ожидание ${id}` });

const FULL = {
  id: "local-runtime",
  title: "Локальный рантайм",
  stage: 1,
  checkpoint: false,
  status: "in_progress",
  hours: { min: 4, max: 6 },
  blocked_by: [],
  verified_at: "2026-07-26",
  outdated: false,
  outcomes: ["Поднимает модель и объясняет расход памяти"],
  misconceptions: ["Квантование бесплатно"],
  materials: [
    {
      title: "llama.cpp README",
      url: "https://example.invalid/llama",
      kind: "repo",
      tier: "T1",
      lang: "en",
      stale: true,
      delta: "флаг --n-gpu-layers переименован",
      offline: "absent",
      note: "",
    },
  ],
  practice: {
    kind: "ops",
    tier: "P2",
    task: "Подними модель локально",
    deliverable: "Лог запуска с числами",
    starting_point: null,
    fallback: "Начни с самой маленькой модели",
    time_box_min: 90,
    smoke_checked: true,
    constraints: [CHECK("c1")],
    acceptance: [CHECK("a1")],
  },
  questions: [{ id: "q1", kind: "diagnose", text: "Что съело память?" }],
  exam: { focus: "расход памяти", artifact_required: true, max_exchanges: 12 },
};

const CHECKPOINT = {
  ...FULL,
  id: "cp-gateway",
  title: "Шлюз к моделям",
  checkpoint: true,
  status: "blocked",
  blocked_by: [{ id: "openai-compatible-api", title: "Совместимый API" }],
  misconceptions: [],
  materials: [],
  questions: [],
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "topic") {
      return options.broken ? Promise.reject(new Error("нет такой")) : Promise.resolve(options.out ?? FULL);
    }
    if (name === "set_status") return Promise.resolve({ status: payload.status });
    throw new Error(`лишняя команда ${name}`);
  };
  render(
    () =>
      Topic({
        text: ru,
        locale: "ru",
        program: options.program ?? "/programs/llm-agents-base",
        topic: options.topic ?? "local-runtime",
        today: "2026-07-27",
        call,
      }),
    host,
  );
  return { host, calls };
}

const section = (host, id) => host.querySelector(`[data-section="${id}"]`);

test("экран читает тему по программе и идентификатору", async () => {
  const { calls } = mount({ program: "/programs/other", topic: "agent-loop" });
  await settled();

  assert.deepEqual(calls, [
    {
      name: "topic",
      payload: { bundle: "/programs/other", topic: "agent-loop", today: "2026-07-27" },
    },
  ]);
});

test("секции идут в порядке работы, а не в порядке полей", async () => {
  const { host } = mount();
  await settled();

  assert.deepEqual(
    [...host.querySelectorAll("[data-section]")].map((node) => node.dataset.section),
    ["outcomes", "misconceptions", "materials", "practice", "questions", "exam", "notes"],
  );
});

test("вырожденный чекпойнт не рисует пустые секции", async () => {
  const { host } = mount({ out: CHECKPOINT });
  await settled();

  assert.deepEqual(
    [...host.querySelectorAll("[data-section]")].map((node) => node.dataset.section),
    ["outcomes", "practice", "exam", "notes"],
  );
});

test("заголовок несёт статус, часы, свежесть и то, чем тема разблокируется", async () => {
  const { host } = mount({ out: CHECKPOINT });
  await settled();

  const header = host.querySelector("[data-header]");
  assert.equal(header.querySelector("[role=img]").getAttribute("aria-label"), ru.status.blocked);
  assert.match(header.textContent, new RegExp(ru.status.blocked));
  assert.match(header.textContent, /4\D+6/);
  assert.match(header.textContent, /Совместимый API/);
});

test("устаревшая тема говорит об этом, свежая молчит", async () => {
  const fresh = mount();
  await settled();
  assert.doesNotMatch(fresh.host.querySelector("[data-header]").textContent, new RegExp(ru.topic.outdated));

  const old = mount({ out: { ...FULL, outdated: true } });
  await settled();
  assert.match(old.host.querySelector("[data-header]").textContent, new RegExp(ru.topic.outdated));
});

test("ограничения и приёмка показаны двумя коллекциями", async () => {
  const { host } = mount();
  await settled();

  const practice = section(host, "practice");
  const constraints = practice.querySelector("[data-constraints]");
  const acceptance = practice.querySelector("[data-acceptance]");
  assert.match(constraints.textContent, /команда c1/);
  assert.match(acceptance.textContent, /команда a1/);
  assert.doesNotMatch(constraints.textContent, /команда a1/, "коллекции слиты");
  assert.match(practice.textContent, new RegExp(ru.topic.constraints));
  assert.match(practice.textContent, new RegExp(ru.topic.acceptance));
});

test("подсказка практики закрыта спойлером, а критерии видны сразу", async () => {
  const { host } = mount();
  await settled();

  const practice = section(host, "practice");
  const hint = practice.querySelector("details");
  assert.match(hint.textContent, /самой маленькой модели/);
  assert.equal(hint.hasAttribute("open"), false);
  assert.equal(practice.querySelector("[data-acceptance] details"), null, "приёмка под спойлером");
});

test("материал несёт свежесть, отличие и офлайн-доступность", async () => {
  const { host } = mount();
  await settled();

  const material = section(host, "materials").querySelector("li");
  assert.match(material.textContent, /llama\.cpp README/);
  assert.match(material.textContent, new RegExp(ru.topic.stale));
  assert.match(material.textContent, /n-gpu-layers/);
  assert.match(material.textContent, new RegExp(ru.topic.offline));
});

test("вопросы показаны формулировками, без ответов", async () => {
  const { host } = mount();
  await settled();

  const questions = section(host, "questions");
  assert.match(questions.textContent, /Что съело память\?/);
  assert.doesNotMatch(host.innerHTML, /expected_signals|red_flags|follow_up/);
});

test("статус ставится вручную и экран обновляется", async () => {
  const { host, calls } = mount();
  await settled();

  const choice = host.querySelector('[data-status-choice="passed"]');
  choice.click();
  await settled();

  assert.deepEqual(calls[1], {
    name: "set_status",
    payload: {
      bundle: "/programs/llm-agents-base",
      topic: "local-runtime",
      status: "passed",
      today: "2026-07-27",
    },
  });
  assert.match(host.querySelector("[data-header]").textContent, new RegExp(ru.status.passed));

  host.querySelector('[data-status-choice="failed"]').click();
  await settled();

  assert.equal(calls[2].payload.status, "failed");
  assert.equal(host.querySelector("[data-status]").textContent, ru.status.failed);
});

test("с темы есть ход на экран практики", async () => {
  const { host } = mount();
  await settled();

  const link = section(host, "practice").querySelector("[data-practice-link]");
  assert.match(link.getAttribute("href"), /\/ru\/practice\/\?program=/);
  assert.match(link.getAttribute("href"), /topic=local-runtime/);
});

test("без темы в адресе экран говорит об этом, а не пустеет", async () => {
  const { host, calls } = mount({ program: "", topic: "" });
  await settled();

  assert.equal(calls.length, 0, "ядро дёрнули без темы");
  assert.match(host.querySelector("[data-empty]").textContent, /Тема не выбрана/);
  assert.equal(host.querySelector("[data-empty] a").getAttribute("href"), "/ru/");
});

test("недоступная тема сообщает о себе", async () => {
  const { host } = mount({ broken: true });
  await settled();

  assert.match(host.querySelector("[data-empty]").textContent, /Тема не выбрана/);
});
