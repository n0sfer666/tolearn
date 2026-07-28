import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Exam;
let render;
let document;

before(
  async () => {
    ({ document } = browser(
      "https://tolearn.local/ru/exam/?program=/programs/llm-agents-base&topic=local-runtime",
    ));
    ({ default: Exam } = await island("Exam"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const PARSED = {
  result: "pass",
  status: "passed",
  gaps: ["квантование"],
  notes: [],
  missing: [],
  unknown_questions: [],
  failed_checks: [],
  per_question: [{ id: "q1", outcome: "ok", missed: [] }],
  hinted: false,
  practice_accepted: true,
  next_action: "proceed",
  retry_after_days: null,
};

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "prompt") return Promise.resolve({ text: "Текст промпта" });
    if (name === "parse_verdict") {
      if (options.broken === true) {
        return Promise.reject(new Error("в ответе нет блока JSON с вердиктом"));
      }
      return Promise.resolve({ ...PARSED, ...(options.parsed ?? {}) });
    }
    if (name === "apply_verdict") {
      return Promise.resolve({
        status: "passed",
        gaps: ["квантование"],
        retry: [],
        split_suggested: options.split === true,
      });
    }
    throw new Error(`лишняя команда ${name}`);
  };
  const copied = [];
  render(
    () =>
      Exam({
        text: ru,
        locale: "ru",
        program: "/programs/llm-agents-base",
        topic: "local-runtime",
        today: "2026-07-28",
        call,
        copy: (text) => {
          if (options.noClipboard === true) return Promise.reject(new Error("нет буфера"));
          copied.push(text);
          return Promise.resolve();
        },
      }),
    host,
  );
  return { host, calls, copied };
}

function paste(host, text) {
  const area = host.querySelector("[data-verdict-input]");
  area.value = text;
  area.dispatchEvent(new document.defaultView.Event("input", { bubbles: true }));
}

test("промпт готов к копированию одним действием", async () => {
  const { host, calls, copied } = mount();
  await settled();

  assert.deepEqual(calls[0], {
    name: "prompt",
    payload: { bundle: "/programs/llm-agents-base", topic: "local-runtime" },
  });
  host.querySelector("[data-copy]").click();
  await settled();

  assert.deepEqual(copied, ["Текст промпта"]);
  assert.match(host.textContent, new RegExp(ru.exam.copied));
});

test("без буфера обмена промпт всё равно виден и его можно выделить", async () => {
  const { host } = mount({ noClipboard: true });
  await settled();

  host.querySelector("[data-copy]").click();
  await settled();

  assert.match(host.querySelector("[data-prompt-text]").textContent, /Текст промпта/);
  assert.match(host.textContent, new RegExp(ru.exam.copyManually));
});

test("разбор идёт по нажатию и ничего не применяет сам", async () => {
  const { host, calls } = mount();
  await settled();

  paste(host, "ответ модели целиком");
  host.querySelector("[data-parse]").click();
  await settled();

  assert.deepEqual(calls[1], {
    name: "parse_verdict",
    payload: {
      bundle: "/programs/llm-agents-base",
      topic: "local-runtime",
      text: "ответ модели целиком",
    },
  });
  assert.equal(calls.length, 2, "вердикт применён без человека");
  assert.match(host.querySelector("[data-parsed]").textContent, new RegExp(ru.status.passed));
});

test("применение уходит тем же текстом ответа, а не выдранным JSON", async () => {
  const { host, calls } = mount();
  await settled();

  paste(host, "проза, а внутри ```json {...} ```");
  host.querySelector("[data-parse]").click();
  await settled();
  host.querySelector("[data-apply]").click();
  await settled();

  assert.deepEqual(calls[2], {
    name: "apply_verdict",
    payload: {
      bundle: "/programs/llm-agents-base",
      topic: "local-runtime",
      text: "проза, а внутри ```json {...} ```",
      today: "2026-07-28",
    },
  });
  assert.match(host.querySelector("[data-applied]").textContent, new RegExp(ru.status.passed));
});

test("пропущенные поля названы, но применение остаётся доступным", async () => {
  const { host } = mount({ parsed: { missing: ["gaps"], unknown_questions: ["q9"] } });
  await settled();

  paste(host, "ответ");
  host.querySelector("[data-parse]").click();
  await settled();

  const missing = host.querySelector("[data-missing]");
  assert.match(missing.textContent, /gaps/);
  assert.match(host.querySelector("[data-unknown]").textContent, /q9/);
  assert.equal(host.querySelector("[data-apply]").disabled, false);
});

test("нераспознанный ответ говорит об этом и не даёт применить", async () => {
  const { host, calls } = mount({ broken: true });
  await settled();

  paste(host, "проза без json");
  host.querySelector("[data-parse]").click();
  await settled();

  assert.match(host.querySelector("[data-broken]").textContent, new RegExp(ru.exam.broken));
  assert.equal(host.querySelector("[data-apply]"), null);
  assert.equal(calls.length, 2);
});

test("три провала подряд ведут на разбор", async () => {
  const { host } = mount({ split: true });
  await settled();

  paste(host, "ответ");
  host.querySelector("[data-parse]").click();
  await settled();
  host.querySelector("[data-apply]").click();
  await settled();

  assert.match(host.querySelector("[data-split]").textContent, new RegExp(ru.exam.split));
  assert.match(host.querySelector("[data-review-link]").getAttribute("href"), /\/ru\/review\/\?program=/);
});
