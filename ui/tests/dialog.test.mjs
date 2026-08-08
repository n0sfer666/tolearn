import assert from "node:assert/strict";
import test, { after, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";

let Dialog;
let render;
let document;

before(async () => {
  ({ document } = browser("https://tolearn.local/ru/exam/dialog/"));
  ({ default: Dialog } = await island("Dialog"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const SHUT = {
  open: false,
  stale: false,
  stage: "practice",
  asked: 0,
  total: 4,
  hint_ready: false,
  log: [],
  graded: [],
  hinted: [],
  seconds: 0,
  tokens: 0,
  verdict: null,
};

const GOING = {
  ...SHUT,
  open: true,
  stage: "question",
  asked: 1,
  hint_ready: true,
  log: [
    { side: "examiner", text: "Откуда взялось 9.8 GB?" },
    { side: "student", text: "Веса, KV-кэш и compute buffers." },
  ],
  seconds: 12,
  tokens: 340,
};

const VERDICT = {
  topic_id: "local-runtime",
  result: "pass",
  status: "passed",
  per_question: [],
  gaps: [],
  missing: [],
  unknown_questions: [],
  notes: [],
  known: true,
  hinted: false,
  date: "2026-08-06",
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
    if (name === "parse_verdict") return Promise.resolve(VERDICT);
    if (name === "apply_verdict") {
      return Promise.resolve({ status: "passed", gaps: [], retry: null, split_suggested: false });
    }
    if (name.startsWith("exam_")) {
      return Promise.resolve(states.length > 1 ? states.shift() : states[0]);
    }
    if (name === "speech_state" || name === "speech_start") {
      return Promise.resolve({
        available: options.voice ?? false,
        listening: name === "speech_start",
        language: "ru",
      });
    }
    if (name === "speech_stop") {
      return options.deaf
        ? Promise.reject({ code: "speech.silent", message: "записи нет" })
        : Promise.resolve({ text: options.heard ?? "Слои разложились на CPU." });
    }
    throw new Error(`лишняя команда ${name}`);
  };
  const dispose = render(
    () =>
      Dialog({
        text: ru,
        locale: "ru",
        program: "/bundle",
        topic: "local-runtime",
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

test("закрытый зачёт предлагает начать, выключенный провайдер уводит в копипаст", async () => {
  const { host } = mount({ enabled: false });
  await ticks();

  assert.equal(host.querySelector("[data-start]").disabled, true);
  assert.match(host.textContent, new RegExp(ru.dialog.off));
  assert.equal(host.querySelector("[data-dialog-off] a").getAttribute("href"), "/ru/exam/?program=%2Fbundle&topic=local-runtime");
});

test("начатый зачёт показывает разговор, счёт вопросов и стоимость", async () => {
  const { host } = mount({ states: [GOING] });
  await ticks();

  const said = [...host.querySelectorAll("[data-talk] li")];
  assert.equal(said.length, 2);
  assert.equal(said[0].getAttribute("data-side"), "examiner");
  assert.match(host.querySelector("[data-count]").textContent, /1 \/ 4/);
  assert.match(host.querySelector("[data-cost]").textContent, /12/);
  assert.match(host.querySelector("[data-cost]").textContent, /340/);
});

test("ответ уходит одной командой и очищает поле", async () => {
  const { host, calls } = mount({ states: [GOING] });
  await ticks();

  const field = host.querySelector("[data-answer]");
  field.value = "Слои разложились на CPU.";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  host.querySelector("[data-say]").click();
  await ticks();

  const [said] = calls.filter(({ name }) => name === "exam_say");
  assert.equal(said.payload.text, "Слои разложились на CPU.");
  assert.equal(said.payload.topic, "local-runtime");
  assert.equal(host.querySelector("[data-answer]").value, "");
});

test("подсказка живёт только на вопросе", async () => {
  const { host, calls } = mount({ states: [GOING] });
  await ticks();

  assert.equal(host.querySelector("[data-hint]").disabled, false);
  host.querySelector("[data-hint]").click();
  await ticks();

  assert.equal(calls.filter(({ name }) => name === "exam_hint").length, 1);
});

test("завершение разбирает вердикт и даёт применить его", async () => {
  const done = { ...GOING, stage: "done", verdict: "```json\n{}\n```" };
  const { host, calls } = mount({ states: [GOING, done] });
  await ticks();

  host.querySelector("[data-finish]").click();
  await ticks(8);

  assert.equal(host.querySelector("[data-reply]"), null);
  assert.equal(calls.filter(({ name }) => name === "parse_verdict").length, 1);

  host.querySelector("[data-apply]").click();
  await ticks();

  const [used] = calls.filter(({ name }) => name === "apply_verdict");
  assert.equal(used.payload.today, "2026-08-06");
  assert.match(host.querySelector("[data-applied]").textContent, new RegExp(ru.exam.applied));
});

test("надиктованное дописывается в конец набранного и правится руками", async () => {
  const { host, calls } = mount({ states: [GOING], voice: true });
  await ticks();

  const field = host.querySelector("[data-answer]");
  field.value = "Веса легли в память.";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();

  const voice = host.querySelector("[data-voice]");
  assert.equal(voice.disabled, false);
  voice.click();
  await ticks();

  assert.equal(host.querySelector("[data-voice]").getAttribute("data-listening"), "true");
  host.querySelector("[data-voice]").click();
  await ticks();

  assert.equal(
    host.querySelector("[data-answer]").value,
    "Веса легли в память. Слои разложились на CPU.",
  );
  assert.equal(host.querySelector("[data-answer]").disabled, false);
  assert.equal(calls.filter(({ name }) => name === "speech_start").length, 1);
  assert.equal(calls.filter(({ name }) => name === "speech_stop").length, 1);
});

test("без распознавания точка входа видна, неактивна и уводит на второй вариант", async () => {
  const { host } = mount({ states: [GOING] });
  await ticks();

  const voice = host.querySelector("[data-voice]");
  assert.notEqual(voice, null);
  assert.equal(voice.disabled, true);
  assert.match(host.querySelector("[data-voice-off]").textContent, /в разработке/);
  assert.equal(
    host.querySelector("[data-voice-variant]").getAttribute("href"),
    "https://github.com/n0sfer666/tolearn/releases",
  );
});

test("отказ распознавания говорит причину и не трогает поле", async () => {
  const said = toasts(document.defaultView);
  const { host } = mount({ states: [GOING], voice: true, deaf: true });
  await ticks();

  const field = host.querySelector("[data-answer]");
  field.value = "Уже набрано.";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  host.querySelector("[data-voice]").click();
  await ticks();
  host.querySelector("[data-voice]").click();
  await ticks();

  assert.equal(host.querySelector("[data-answer]").value, "Уже набрано.");
  assert.equal(said.at(-1).tone, "error");
  assert.match(said.at(-1).text, /записи нет/);
});

test("устаревшая тема видна и перезапуск идёт с флагом", async () => {
  const { host, calls } = mount({ states: [{ ...GOING, stale: true }] });
  await ticks();

  assert.match(host.querySelector("[data-stale]").textContent, new RegExp(ru.dialog.stale));
  host.querySelector("[data-restart]").click();
  await ticks();

  const [again] = calls.filter(({ name }) => name === "exam_start");
  assert.equal(again.payload.restart, true);
});
