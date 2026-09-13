import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";
import { deferred, heard, named, press, refusal, rejected, sequence, transport } from "./support/generation.mjs";

let Stage;
let render;
let window;

before(
  async () => {
    window = browser("https://tolearn.local/ru/stage/?program=chip&node=rom&stage=voices");
    globalThis.location = window.location;
    ({ default: Stage } = await island("Stage"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const paragraph = (text) => ({
  id: "p1",
  kind: "paragraph",
  text,
  lang: null,
  src: null,
  license: null,
  attribution: null,
  source: null,
});

const OUT = {
  program: "chip",
  node: "rom",
  node_title: "Первый ROM",
  id: "voices",
  title: "Голоса чипа",
  blocks: [paragraph("Прежний текст этапа")],
  practice: { task: [], deliverable: "", constraints: [], acceptance: [] },
  questions: [],
  ticks: [],
  workdir: null,
  clarifications: [],
};
const AGAIN = { ...OUT, blocks: [paragraph("Новый текст этапа")] };
const focused = () => window.document.activeElement;

function mount(options = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const shown = [options.out ?? OUT, AGAIN];
  const { calls, call } = transport({
    stage: () => Promise.resolve(shown.shift() ?? AGAIN),
    regenerate_stage: { stage: "voices" },
    cancel_generation: { cancelled: true },
    speech_state: { available: false, listening: false, language: "ru" },
    ...options.answers,
  });
  const said = toasts(window);
  const { steps, emit } = heard();
  const dispose = render(() => Stage({ text: ru, locale: "ru", call, steps }), host);
  return { host, calls, said, dispose, emit };
}

const body = (host) => host.querySelector("[data-stage-body]").textContent;

test("в конце этапа — «Пропустить зачёт» к развилке и «Перегенерировать этап»", async () => {
  const { host } = mount();
  await settled();

  const skip = host.querySelector("[data-next]");
  assert.equal(skip.getAttribute("href"), "/ru/next/?program=chip&node=rom&stage=voices");
  assert.equal(skip.textContent, ru.generate.skip);
  assert.equal(host.querySelector("[data-regenerate]").textContent, ru.generate.regenerate);

  const root = mount({ out: { ...OUT, node: "chip" } });
  await settled();
  assert.equal(root.host.querySelector("[data-next]").getAttribute("href"), "/ru/next/?program=chip&stage=voices");
});

test("перегенерация держит прежний этап на экране, подменяет его по готовности и возвращает фокус", async () => {
  const again = deferred();
  const { host, calls, said, emit } = mount({ answers: { regenerate_stage: again.answer } });
  await settled();
  press(host, "[data-regenerate]");
  await settled();
  emit({ step: "text", state: "began", round: 0, of: 0 });

  assert.match(body(host), /Прежний текст этапа/);
  assert.equal(host.querySelector("[data-progress] [data-step]").textContent, ru.generate.stepText);
  assert.ok(host.querySelector("[data-regenerate]") === null, "перегенерацию можно запустить второй раз");
  assert.deepEqual(named(calls, "regenerate_stage")[0].payload, { program: "chip", node: "rom", stage: "voices" });

  again.resolve({ stage: "voices" });
  await settled();
  await settled();
  assert.match(body(host), /Новый текст этапа/);
  assert.deepEqual(said.at(-1), { tone: "ok", text: ru.generate.regenerated });
  assert.ok(host.querySelector("[data-progress]") === null);
  assert.ok(focused() === host.querySelector("[data-regenerate]"), "фокус не вернулся на кнопку");
});

test("отмена перегенерации оставляет прежний этап и возвращает фокус на кнопку", async () => {
  const again = deferred();
  const { host, calls, said } = mount({ answers: { regenerate_stage: again.answer } });
  await settled();
  press(host, "[data-regenerate]");
  await settled();
  press(host, "[data-cancel]");
  again.reject(refusal("generate.cancelled", "отменено"));
  await settled();

  assert.equal(named(calls, "cancel_generation").length, 1);
  assert.equal(named(calls, "stage").length, 1);
  assert.match(body(host), /Прежний текст этапа/);
  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.cancelled });
  assert.ok(focused() === host.querySelector("[data-regenerate]"), "фокус не вернулся на кнопку");
});

test("не прочитавшийся новый этап оставляет прежний на экране и говорит об этом тостом", async () => {
  const stage = sequence(() => Promise.resolve(OUT), rejected(refusal("stage.absent", "stage is gone")));
  const { host, said } = mount({ answers: { stage } });
  await settled();
  press(host, "[data-regenerate]");
  await settled();
  await settled();

  assert.match(body(host), /Прежний текст этапа/);
  assert.deepEqual(said.at(-1), { tone: "error", text: ru.generate.unread });
  assert.ok(host.querySelector("[data-empty]") === null, "этап заменён пустым экраном");
});

test("отказ перегенерации называет причину и ведёт в настройки, этап остаётся", async () => {
  const failed = rejected(refusal("generate.unrepaired", "модель не справилась с правкой"));
  const { host } = mount({ answers: { regenerate_stage: failed } });
  await settled();
  press(host, "[data-regenerate]");
  await settled();

  const refused = host.querySelector("[data-refused]");
  assert.match(refused.textContent, /модель не справилась с правкой/);
  assert.equal(refused.querySelector("[data-to-settings]").getAttribute("href"), "/ru/settings/");
  assert.match(body(host), /Прежний текст этапа/);
});
