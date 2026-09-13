import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled, toasts } from "./support/dom.mjs";
import { began, deferred, heard, named, press, refusal, rejected, sequence } from "./support/generation.mjs";
import { OUT, stageScreen } from "./support/stage.mjs";

const { screen, mount } = stageScreen();

const PLACE = { program: "chip", node: "chip", stage: "voices" };
const DRAFTED = {
  ...OUT,
  questions: [
    { ...OUT.questions[0], draft: "Пять" },
    { ...OUT.questions[1], draft: "" },
  ],
};
const GRADED = {
  ...OUT,
  questions: [
    { ...OUT.questions[0], draft: "Пять", result: "ok" },
    { ...OUT.questions[1], draft: "", result: "miss", missed: ["форма волны"] },
  ],
};

const field = (host, id) => host.querySelector(`#${id} textarea[data-answer]`);
const pause = () => new Promise((resolve) => setTimeout(resolve, 450));

function type(host, id, text) {
  const area = field(host, id);
  area.value = text;
  area.dispatchEvent(new screen.window.Event("input", { bubbles: true }));
  return area;
}

function sit(options = {}) {
  const { steps, emit } = heard();
  const said = toasts(screen.window);
  const mounted = mount({
    out: options.out ?? DRAFTED,
    props: { steps },
    replies: { answer: () => ({}), cancel_generation: () => ({ cancelled: true }), ...options.replies },
  });
  return { ...mounted, emit, said };
}

test("под каждым вопросом стоит поле ответа со своим черновиком", async () => {
  const { host, calls } = sit();
  await settled();

  assert.equal(field(host, "q1").value, "Пять");
  assert.equal(field(host, "q2").value, "");
  assert.match(host.querySelector("#q1 label").textContent, new RegExp(ru.stage.answer));
  assert.equal(host.querySelector("[data-exam]").textContent, ru.stage.submit);
  assert.deepEqual(
    calls.map((made) => made.name),
    ["stage"],
  );
});

test("черновик пишется при уходе из поля, после паузы и при закрытии окна", async () => {
  const { host, calls } = sit();
  await settled();

  type(host, "q2", "Треугольник").dispatchEvent(new screen.window.Event("blur"));
  await settled();
  assert.deepEqual(named(calls, "answer").at(-1).payload, { ...PLACE, question: "q2", text: "Треугольник" });

  field(host, "q1").dispatchEvent(new screen.window.Event("blur"));
  await settled();
  assert.equal(named(calls, "answer").length, 1, "нетронутое поле ушло в состояние");

  type(host, "q1", "Пять каналов");
  await settled();
  assert.equal(named(calls, "answer").length, 1, "черновик ушёл до паузы");
  await pause();
  assert.deepEqual(named(calls, "answer").at(-1).payload, { ...PLACE, question: "q1", text: "Пять каналов" });

  type(host, "q2", "");
  screen.window.dispatchEvent(new screen.window.Event("pagehide"));
  await settled();
  assert.deepEqual(named(calls, "answer").at(-1).payload, { ...PLACE, question: "q2", text: "" });
});

test("«Сдать» дописывает черновик и уходит одним запросом, итог встаёт у вопросов", async () => {
  const exam = deferred();
  const stage = sequence(
    () => DRAFTED,
    () => GRADED,
  );
  const { host, calls, emit, said } = sit({ replies: { exam: exam.answer, stage } });
  await settled();

  type(host, "q2", "Форма");
  press(host, "[data-exam]");
  await settled();
  await settled();
  emit(began("exam"));

  assert.deepEqual(
    calls.map((made) => made.name),
    ["stage", "answer", "exam"],
  );
  assert.deepEqual(named(calls, "exam")[0].payload, {
    ...PLACE,
    answers: [
      { id: "q1", text: "Пять" },
      { id: "q2", text: "Форма" },
    ],
  });
  assert.equal(host.querySelector("[data-progress] [data-step]").textContent, ru.generate.stepExam);
  assert.ok(host.querySelector("[data-progress] [data-cancel]"));
  assert.equal(host.querySelector("[data-exam]"), null, "зачёт можно сдать второй раз");
  assert.equal(field(host, "q1").readOnly, true, "ответ правится во время проверки");

  exam.resolve({ passed: false });
  await settled();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "info", text: ru.stage.graded });
  const q2 = host.querySelector("#q2");
  assert.equal(q2.dataset.result, "miss");
  assert.deepEqual(
    [...q2.querySelectorAll("[data-missed] li")].map((node) => node.textContent),
    ["форма волны"],
  );
  assert.equal(field(host, "q1").readOnly, false);
  assert.ok(screen.window.document.activeElement === host.querySelector("[data-exam]"), "фокус не вернулся");
});

test("сданный зачёт говорит об этом", async () => {
  const { host, said } = sit({ replies: { exam: () => ({ passed: true }) } });
  await settled();

  press(host, "[data-exam]");
  await settled();
  await settled();

  assert.deepEqual(said.at(-1), { tone: "ok", text: ru.stage.passed });
});

test("пустой зачёт не уходит и говорит, почему", async () => {
  const { host, calls } = sit({ out: OUT, replies: { exam: () => ({ passed: true }) } });
  await settled();

  type(host, "q1", "  \n");
  press(host, "[data-exam]");
  await settled();

  assert.equal(named(calls, "exam").length, 0);
  const refused = host.querySelector("[data-refused]");
  assert.match(refused.textContent, new RegExp(ru.stage.blank));
  assert.equal(refused.querySelector("[data-to-settings]"), null);
});

test("отказ проверки называет причину, ответы остаются в полях", async () => {
  const failed = rejected(refusal("generate.verdict", "модель не прислала годный вердикт и после починки"));
  const { host, calls } = sit({ replies: { exam: failed } });
  await settled();

  type(host, "q2", "Форма");
  press(host, "[data-exam]");
  await settled();
  await settled();

  const refused = host.querySelector("[data-refused]");
  assert.match(refused.textContent, /не прислала годный вердикт/);
  assert.equal(refused.querySelector("[data-to-settings]").getAttribute("href"), "/ru/settings/");
  assert.equal(field(host, "q1").value, "Пять");
  assert.equal(field(host, "q2").value, "Форма");
  assert.equal(field(host, "q2").readOnly, false);
  assert.equal(named(calls, "stage").length, 1, "этап перечитан после отказа");
});
