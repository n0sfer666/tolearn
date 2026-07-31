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
      title: "Гайд по квантованию",
      url: "https://example.invalid/quant",
      kind: "doc",
      tier: "T1",
      lang: "ru",
      stale: false,
      delta: null,
      offline: "absent",
      note: "Отсюда начинается тема: как считается расход памяти",
    },
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
  unload: { state: "missing", checked_at: null },
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
    if (name === "note") return Promise.resolve({ body: "первая строка", stamp: null, path: "" });
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

  assert.deepEqual(calls[0], {
    name: "topic",
    payload: { bundle: "/programs/other", topic: "agent-loop", today: "2026-07-27" },
  });
});

test("секции идут в порядке работы, а не в порядке полей", async () => {
  const { host } = mount();
  await settled();

  assert.deepEqual(
    [...host.querySelectorAll("[data-section]")].map((node) => node.dataset.section),
    ["materials", "outcomes", "exam", "questions", "misconceptions"],
  );
});

test("второстепенное убрано под раскрытие, материалы и итоги — нет", async () => {
  const { host } = mount();
  await settled();

  assert.deepEqual(
    [...host.querySelectorAll("details[data-section]")].map((node) => node.dataset.section),
    ["exam", "questions", "misconceptions"],
  );
});

test("вырожденный чекпойнт не рисует пустые секции", async () => {
  const { host } = mount({ out: CHECKPOINT });
  await settled();

  assert.deepEqual(
    [...host.querySelectorAll("[data-section]")].map((node) => node.dataset.section),
    ["outcomes", "exam"],
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

test("неактуальные материалы скрыты, пока свитч выключен", async () => {
  const { host } = mount();
  await settled();

  const materials = section(host, "materials");
  const titles = () => [...materials.querySelectorAll("li a")].map((node) => node.textContent);

  assert.deepEqual(titles(), ["Гайд по квантованию"]);

  materials.querySelector("[data-show-stale]").click();
  await settled();

  assert.deepEqual(titles(), ["Гайд по квантованию", "llama.cpp README"]);
});

test("материал несёт свежесть, отличие и офлайн-доступность", async () => {
  const { host } = mount();
  await settled();

  section(host, "materials").querySelector("[data-show-stale]").click();
  await settled();

  const material = section(host, "materials").querySelector("li[data-material='repo']");
  assert.match(material.textContent, /llama\.cpp README/);
  assert.match(material.textContent, new RegExp(ru.topic.stale));
  assert.match(material.textContent, /n-gpu-layers/);
  assert.match(material.textContent, new RegExp(ru.topic.offline));
});

test("материалы идут путём по порядку и говорят, зачем их читать", async () => {
  const { host } = mount();
  await settled();

  const materials = section(host, "materials");

  assert.ok(materials.querySelector("ol[data-materials]"), materials.innerHTML);
  assert.equal(materials.querySelector("ul"), null, "путь читается порядком, а не кучей");
  assert.match(
    materials.querySelector("li[data-material='doc'] [data-note]").textContent,
    /расход памяти/,
  );
});

test("вопросы показаны формулировками, без ответов", async () => {
  const { host } = mount();
  await settled();

  const questions = section(host, "questions");
  assert.match(questions.textContent, /Что съело память\?/);
  assert.doesNotMatch(host.innerHTML, /expected_signals|red_flags|follow_up/);
});

test("статус переключается по шагам вперёд и назад", async () => {
  const { host, calls } = mount();
  await settled();

  const step = (way) => host.querySelector(`[data-step="${way}"]`);
  assert.equal(step("next").textContent, ru.steps.toExam);
  assert.equal(step("back").textContent, ru.steps.toStart);

  step("next").click();
  await settled();

  const marks = () => calls.filter((made) => made.name === "set_status");
  assert.deepEqual(marks().at(-1), {
    name: "set_status",
    payload: {
      bundle: "/programs/llm-agents-base",
      topic: "local-runtime",
      status: "exam_pending",
      today: "2026-07-27",
    },
  });
  assert.equal(host.querySelector("[data-status]").textContent, ru.status.exam_pending);
  assert.equal(step("next").textContent, ru.steps.pass);

  step("back").click();
  await settled();

  assert.equal(marks().at(-1).payload.status, "in_progress");
  assert.equal(host.querySelector("[data-status]").textContent, ru.status.in_progress);
});

test("из «не начата» назад хода нет, а провал зовёт начать заново", async () => {
  const fresh = mount({ out: { ...FULL, status: "todo" } });
  await settled();

  assert.equal(fresh.host.querySelector('[data-step="back"]'), null, "из «не начата» есть назад");
  assert.equal(fresh.host.querySelector('[data-step="next"]').textContent, ru.steps.start);

  const broken = mount({ out: { ...FULL, status: "failed" } });
  await settled();

  assert.equal(broken.host.querySelector('[data-step="back"]'), null, "у провала есть назад");
  assert.equal(broken.host.querySelector('[data-step="next"]').textContent, ru.steps.restart);
});

test("выведенный статус руками не двигается", async () => {
  const { host } = mount({ out: CHECKPOINT, topic: "cp-gateway" });
  await settled();

  assert.equal(host.querySelector("[data-step]"), null, "заблокированную тему двигают руками");
  assert.equal(host.querySelector("[data-steps-locked]").textContent, ru.steps.locked);
});

test("конспект живёт в панели, которую открывает и закрывает фаб", async () => {
  const { host } = mount();
  await settled();

  assert.equal(host.querySelector("[data-note-panel]"), null, "панель лезет на глаза сразу");
  assert.equal(host.querySelector("[data-note-fab]").getAttribute("aria-expanded"), "false");

  host.querySelector("[data-note-fab]").click();
  await settled();

  assert.ok(host.querySelector("[data-note-panel] [data-note]"), "фаб не открыл конспект");

  host.querySelector("[data-note-close]").click();
  await settled();

  assert.equal(host.querySelector("[data-note-panel]"), null, "панель не убралась");
});

test("с темы есть ход на практику и на зачёт", async () => {
  const { host } = mount();
  await settled();

  const practice = host.querySelector("[data-tools] [data-practice-link]");
  assert.match(practice.getAttribute("href"), /\/ru\/practice\/\?program=/);
  assert.match(practice.getAttribute("href"), /topic=local-runtime/);

  const exam = host.querySelector("[data-tools] [data-exam-link]");
  assert.match(exam.getAttribute("href"), /\/ru\/exam\/\?program=/);
  assert.match(exam.getAttribute("href"), /topic=local-runtime/);
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
