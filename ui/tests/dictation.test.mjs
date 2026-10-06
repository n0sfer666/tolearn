import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { spliced } from "../src/lib/dictation.ts";
import { settled, toasts } from "./support/dom.mjs";
import { heard, named, press, refusal, rejected } from "./support/generation.mjs";
import { selects } from "./support/pick.mjs";
import { OUT, stageScreen } from "./support/stage.mjs";

const { screen, mount } = stageScreen();

const PLACE = { program: "chip", node: "chip", stage: "voices" };
const LIVE = { available: true, listening: false, language: "ru" };

const rounds = async (times = 4) => {
  for (let round = 0; round < times; round += 1) await settled();
};
const pause = () => new Promise((resolve) => setTimeout(resolve, 450));

const clarifying = async (host) => {
  selects(screen.window).focus(host, "p1");
  await settled();
  press(host, "[data-pick='p1']");
  await settled();
};

function open(options = {}) {
  const said = toasts(screen.window);
  const mounted = mount({
    out: options.out ?? OUT,
    speech: options.speech ?? LIVE,
    props: { steps: heard().steps },
    replies: {
      answer: () => ({}),
      speech_start: () => LIVE,
      speech_stop: () => ({ text: "звуковых" }),
      ...options.replies,
    },
  });
  return { ...mounted, said };
}

function write(area, value) {
  area.value = value;
  area.dispatchEvent(new screen.window.Event("input", { bubbles: true }));
}

const dictated = async (host, scope) => {
  press(host, `${scope} [data-dictate]`);
  await rounds();
  press(host, `${scope} [data-dictate]`);
  await rounds();
};

test("вставка встаёт в позицию курсора с одним пробелом по краям", () => {
  assert.deepEqual(spliced("", 0, 0, "  Пять "), { text: "Пять", caret: 4 });
  assert.deepEqual(spliced("Пять каналов", 4, 4, "звуковых"), { text: "Пять звуковых каналов", caret: 13 });
  assert.deepEqual(spliced("Пять", 4, 4, "каналов"), { text: "Пять каналов", caret: 12 });
  assert.deepEqual(spliced("Их .", 3, 3, "пять"), { text: "Их пять.", caret: 7 });
  assert.deepEqual(spliced("Пять старых каналов", 5, 11, "новых"), { text: "Пять новых каналов", caret: 10 });
  assert.deepEqual(spliced("Пять", 2, 2, "  "), { text: "Пять", caret: 2 });
});

test("в базовой сборке кнопки диктовки нет ни у одного поля", async () => {
  const { host, probes } = open({ speech: { available: false } });
  await rounds();
  await clarifying(host);

  assert.deepEqual(probes, [{ program: "chip" }]);
  assert.ok(host.querySelector("[data-questions] textarea[data-answer]") !== null, "поля ответа нет");
  assert.ok(host.querySelector("[data-clarify='p1'] textarea[data-doubt]") !== null, "поля вопроса нет");
  assert.ok(host.querySelector("[data-dictate]") === null, "кнопка диктовки в базовой сборке");
});

test("запись, оставшаяся от прошлой страницы, глушится при открытии этапа", async () => {
  const { calls } = open({ speech: { ...LIVE, listening: true } });
  await rounds();

  assert.deepEqual(
    named(calls, "speech_stop").map((made) => made.payload),
    [{ program: "chip" }],
  );
});

test("в сборке с речью кнопка стоит у ответа зачёта, у вопроса и у продолжения цепочки", async () => {
  const chain = { chain: 0, block: "p1", excerpt: "Чип держит", fragment: null, turns: [{ asked: null, answer: "Иначе" }], clear: false };
  const { host } = open({ out: { ...OUT, clarifications: [chain] } });
  await rounds();
  await clarifying(host);
  press(host, "[data-clarify='p1'] [data-chain] [data-no]");
  await settled();

  assert.equal(host.querySelectorAll("li[data-question] [data-dictate]").length, 2);
  assert.ok(host.querySelector("[data-clarify='p1'] > [data-asking] [data-dictate]") !== null, "нет у вопроса");
  assert.ok(host.querySelector("[data-clarify='p1'] [data-chain] [data-asking] [data-dictate]") !== null, "нет у цепочки");
  assert.equal(host.querySelector("#q1 [data-dictate]").textContent, ru.stage.dictate);
});

test("надиктованное встаёт в позицию курсора ответа, правится руками и уходит в черновик", async () => {
  const { host, calls } = open();
  await rounds();
  const area = host.querySelector("#q1 textarea[data-answer]");
  write(area, "Пять каналов");
  area.setSelectionRange(4, 4);

  press(host, "#q1 [data-dictate]");
  await rounds();
  assert.deepEqual(named(calls, "speech_start")[0].payload, { program: "chip" });
  assert.equal(host.querySelector("#q1 [data-dictate]").getAttribute("data-listening"), "true");
  assert.equal(host.querySelector("#q1 [data-dictate]").textContent, ru.stage.hush);
  assert.equal(host.querySelector("#q2 [data-dictate]").disabled, true, "второй микрофон открывается");

  press(host, "#q1 [data-dictate]");
  await rounds();
  assert.deepEqual(named(calls, "speech_stop")[0].payload, { program: "chip" });
  assert.equal(area.value, "Пять звуковых каналов");
  assert.equal(area.selectionStart, 13);
  assert.equal(area.readOnly, false);
  assert.equal(host.querySelector("#q1 [data-dictate]").textContent, ru.stage.dictate);
  assert.equal(host.querySelector("#q2 [data-dictate]").disabled, false);

  await pause();
  assert.deepEqual(named(calls, "answer").at(-1).payload, { ...PLACE, question: "q1", text: "Пять звуковых каналов" });
});

test("надиктованный вопрос к «Уточнить» уходит тем текстом, что стоит в поле", async () => {
  const clarify = () => ({ clarifications: [] });
  const { host, calls } = open({ replies: { clarify, speech_stop: () => ({ text: "пять?" }) } });
  await rounds();
  await clarifying(host);
  write(host.querySelector("[data-clarify='p1'] textarea[data-doubt]"), "Почему");

  await dictated(host, "[data-clarify='p1'] [data-asking]");
  assert.equal(host.querySelector("[data-clarify='p1'] textarea[data-doubt]").value, "Почему пять?");
  press(host, "[data-clarify='p1'] [data-ask]");
  await rounds();

  assert.equal(named(calls, "clarify")[0].payload.question, "Почему пять?");
});

test("отказ микрофона называет причину и не трогает поле", async () => {
  const deaf = rejected(refusal("speech.deaf", "микрофон не открылся: нет доступа"));
  const { host, said } = open({ replies: { speech_start: deaf } });
  await rounds();
  const area = host.querySelector("#q1 textarea[data-answer]");
  write(area, "Уже набрано");

  press(host, "#q1 [data-dictate]");
  await rounds();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.stage.speechDeaf });
  assert.equal(area.value, "Уже набрано");
  assert.equal(host.querySelector("#q1 [data-dictate]").textContent, ru.stage.dictate);
  assert.equal(host.querySelector("#q2 [data-dictate]").disabled, false);
});

test("пустое распознавание говорит об этом и не трогает поле", async () => {
  const { host, said } = open({ replies: { speech_stop: () => ({ text: "  " }) } });
  await rounds();
  const area = host.querySelector("#q1 textarea[data-answer]");
  write(area, "Уже набрано");

  await dictated(host, "#q1");

  assert.deepEqual(said.at(-1), { tone: "info", text: ru.stage.unheard });
  assert.equal(area.value, "Уже набрано");
});

test("закрытое поле вопроса выключает микрофон", async () => {
  const { host, calls } = open();
  await rounds();
  await clarifying(host);
  press(host, "[data-clarify='p1'] [data-asking] [data-dictate]");
  await rounds();

  press(host, "[data-clarify='p1'] [data-unask]");
  await rounds();

  assert.equal(named(calls, "speech_stop").length, 1);
  assert.equal(host.querySelector("#q1 [data-dictate]").disabled, false, "микрофон остался занят");
});
