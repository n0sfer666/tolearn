import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

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
    if (name === "provider") {
      return Promise.resolve({
        provider: {
          enabled: options.provider === true,
          flavor: "ollama",
          endpoint: "http://127.0.0.1:11434",
          model: "llama3:8b",
        },
        has_key: false,
        checked: null,
      });
    }
    if (name === "examine") {
      if (options.refuse !== undefined) return Promise.reject({ code: options.refuse });
      return Promise.resolve({ text: options.said ?? "ответ модели целиком" });
    }
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
  const said = toasts(document.defaultView);
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
  const only = (name) => calls.filter((made) => made.name === name);
  return { host, calls, copied, only, said };
}

function paste(host, text) {
  const area = host.querySelector("[data-verdict-input]");
  area.value = text;
  area.dispatchEvent(new document.defaultView.Event("input", { bubbles: true }));
}

test("промпт готов к копированию одним действием", async () => {
  const { host, only, copied, said } = mount();
  await settled();

  assert.deepEqual(only("prompt")[0], {
    name: "prompt",
    payload: { bundle: "/programs/llm-agents-base", topic: "local-runtime" },
  });
  host.querySelector("[data-copy]").click();
  await settled();

  assert.deepEqual(copied, ["Текст промпта"]);
  assert.deepEqual(said.at(-1), { tone: "ok", text: ru.exam.copied });
});

test("без буфера обмена промпт всё равно виден и его можно выделить", async () => {
  const { host, said } = mount({ noClipboard: true });
  await settled();

  host.querySelector("[data-copy]").click();
  await settled();

  assert.match(host.querySelector("[data-prompt-text]").textContent, /Текст промпта/);
  assert.deepEqual(said.at(-1), { tone: "warn", text: ru.exam.copyManually });
});

test("разбор идёт по нажатию и ничего не применяет сам", async () => {
  const { host, only } = mount();
  await settled();

  paste(host, "ответ модели целиком");
  host.querySelector("[data-parse]").click();
  await settled();

  assert.deepEqual(only("parse_verdict")[0], {
    name: "parse_verdict",
    payload: {
      bundle: "/programs/llm-agents-base",
      topic: "local-runtime",
      text: "ответ модели целиком",
    },
  });
  assert.equal(only("apply_verdict").length, 0, "вердикт применён без человека");
  assert.match(host.querySelector("[data-parsed]").textContent, new RegExp(ru.status.passed));
});

test("применение уходит тем же текстом ответа, а не выдранным JSON", async () => {
  const { host, only } = mount();
  await settled();

  paste(host, "проза, а внутри ```json {...} ```");
  host.querySelector("[data-parse]").click();
  await settled();
  host.querySelector("[data-apply]").click();
  await settled();

  assert.deepEqual(only("apply_verdict")[0], {
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
  const { host, only, said } = mount({ broken: true });
  await settled();

  paste(host, "проза без json");
  host.querySelector("[data-parse]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.exam.broken });
  assert.equal(host.querySelector("[data-apply]"), null);
  assert.equal(only("apply_verdict").length, 0);
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

test("включённый провайдер спрашивает модель прямо здесь", async () => {
  const { host, only } = mount({ provider: true });
  await settled();

  host.querySelector("[data-ask]").click();
  await settled();

  assert.deepEqual(only("examine")[0].payload, {
    bundle: "/programs/llm-agents-base",
    topic: "local-runtime",
  });
  assert.equal(host.querySelector("[data-verdict-input]").value, "ответ модели целиком");
  assert.equal(only("parse_verdict")[0].payload.text, "ответ модели целиком");
  assert.match(host.querySelector("[data-parsed]").textContent, new RegExp(ru.status.passed));
});

test("ответ модели правится руками до применения", async () => {
  const { host, only } = mount({ provider: true });
  await settled();

  host.querySelector("[data-ask]").click();
  await settled();
  paste(host, "правленый ответ");
  host.querySelector("[data-apply]").click();
  await settled();

  assert.equal(only("apply_verdict")[0].payload.text, "правленый ответ");
});

test("выключенный провайдер оставляет копипаст-цикл", async () => {
  const { host, only } = mount();
  await settled();

  assert.equal(host.querySelector("[data-ask]"), null);
  paste(host, "ответ вставлен руками");
  host.querySelector("[data-parse]").click();
  await settled();

  assert.equal(only("examine").length, 0);
  assert.equal(only("parse_verdict")[0].payload.text, "ответ вставлен руками");
  assert.match(host.querySelector("[data-parsed]").textContent, new RegExp(ru.status.passed));
});

test("отказ провайдера объясняется словами и не стирает вставленное", async () => {
  const { host, only, said } = mount({ provider: true, refuse: "provider.unreachable" });
  await settled();

  paste(host, "вставлено руками");
  host.querySelector("[data-ask]").click();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.provider.unreachable });
  assert.equal(host.querySelector("[data-verdict-input]").value, "вставлено руками");
  assert.equal(only("parse_verdict").length, 0);
});
