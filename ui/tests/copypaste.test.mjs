import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled, toasts } from "./support/dom.mjs";
import { heard, named, press, refusal, rejected, sequence } from "./support/generation.mjs";
import { OUT, stageScreen } from "./support/stage.mjs";

const { screen, mount } = stageScreen();

const PLACE = { program: "chip", node: "chip", stage: "voices" };
const PROMPT = "Ты принимаешь письменный зачёт по этапу учебной программы «Голоса чипа»";
const REPLY = 'Разбор.\n\n```json\n{"stage": "voices", "per_question": []}\n```';
const DRAFTED = {
  ...OUT,
  questions: [
    { ...OUT.questions[0], draft: "Пять" },
    { ...OUT.questions[1], draft: "" },
  ],
};
const GRADED = {
  ...DRAFTED,
  questions: [
    { ...DRAFTED.questions[0], result: "ok" },
    { ...DRAFTED.questions[1], result: "miss", missed: ["форма волны"] },
  ],
};

function write(host, selector, text) {
  const area = host.querySelector(selector);
  area.value = text;
  area.dispatchEvent(new screen.window.Event("input", { bubbles: true }));
}

function sit(options = {}) {
  const { steps } = heard();
  const said = toasts(screen.window);
  const copied = [];
  const kept = (text) => {
    copied.push(text);
    return Promise.resolve();
  };
  const mounted = mount({
    out: options.out ?? DRAFTED,
    props: { steps, copy: options.copy ?? kept },
    replies: { answer: () => ({}), exam_prompt: () => ({ prompt: PROMPT }), ...options.replies },
  });
  return { ...mounted, said, copied };
}

const refused = (host) => host.querySelector("[data-copypaste] [data-refused]");

test("«Скопировать промпт» кладёт в буфер промпт зачёта с ответами из полей", async () => {
  const { host, calls, said, copied } = sit();
  await settled();

  write(host, "#q2 textarea[data-answer]", "Форма");
  press(host, "[data-copypaste] [data-prompt]");
  await settled();
  await settled();

  assert.deepEqual(
    calls.map((made) => made.name),
    ["stage", "answer", "exam_prompt"],
  );
  assert.deepEqual(named(calls, "exam_prompt")[0].payload, {
    ...PLACE,
    answers: [
      { id: "q1", text: "Пять" },
      { id: "q2", text: "Форма" },
    ],
  });
  assert.deepEqual(copied, [PROMPT]);
  assert.deepEqual(said.at(-1), { tone: "ok", text: ru.stage.prompted });
  assert.equal(host.querySelector("[data-prompt-text]"), null);
});

test("без буфера промпт встаёт в поле, откуда его копируют руками", async () => {
  const { host, said } = sit({ copy: () => Promise.reject(new Error("буфера нет")) });
  await settled();

  press(host, "[data-copypaste] [data-prompt]");
  await settled();
  await settled();

  assert.equal(host.querySelector("[data-prompt-text]").value, PROMPT);
  assert.deepEqual(said.at(-1), { tone: "warn", text: ru.stage.promptManual });
});

test("без ответов и без вставки ничего не уходит, причина видна", async () => {
  const { host, calls } = sit({ out: OUT, replies: { exam_paste: () => ({ passed: true }) } });
  await settled();

  press(host, "[data-copypaste] [data-prompt]");
  await settled();
  assert.match(refused(host).textContent, new RegExp(ru.stage.blank));

  write(host, "[data-copypaste] textarea[data-paste]", "  \n");
  press(host, "[data-copypaste] [data-apply]");
  await settled();
  assert.match(refused(host).textContent, new RegExp(ru.stage.pasteBlank));

  assert.deepEqual(
    calls.map((made) => made.name),
    ["stage"],
  );
});

test("вставленный ответ чата применяет вердикт, итог встаёт у вопросов", async () => {
  const stage = sequence(
    () => DRAFTED,
    () => GRADED,
  );
  const { host, calls, said } = sit({ replies: { stage, exam_paste: () => ({ passed: false }) } });
  await settled();

  write(host, "[data-copypaste] textarea[data-paste]", REPLY);
  press(host, "[data-copypaste] [data-apply]");
  await settled();
  await settled();

  assert.deepEqual(named(calls, "exam_paste")[0].payload, { ...PLACE, text: REPLY });
  assert.deepEqual(said.at(-1), { tone: "info", text: ru.stage.graded });
  assert.equal(host.querySelector("#q2").dataset.result, "miss");
  assert.equal(host.querySelector("[data-copypaste] textarea[data-paste]").value, "");
  assert.equal(named(calls, "stage").length, 2);
});

test("неразобранная вставка называет причину, этап не перечитан, текст остаётся", async () => {
  const failed = rejected(refusal("exam.verdict", "в тексте нет JSON-блока с вердиктом"));
  const { host, calls } = sit({ replies: { exam_paste: failed } });
  await settled();

  write(host, "[data-copypaste] textarea[data-paste]", "Всё отлично");
  press(host, "[data-copypaste] [data-apply]");
  await settled();
  await settled();

  assert.match(refused(host).textContent, /нет JSON-блока/);
  assert.equal(refused(host).querySelector("[data-to-settings]"), null);
  assert.equal(host.querySelector("[data-copypaste] textarea[data-paste]").value, "Всё отлично");
  assert.equal(host.querySelector("[data-copypaste] [data-apply]").disabled, false);
  assert.equal(named(calls, "stage").length, 1);
});
