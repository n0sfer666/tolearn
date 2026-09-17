import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { began, deferred, held, named, press, refusal, rejected } from "./support/generation.mjs";
import { REQUEST, STARTED, fill, newScreen } from "./support/new.mjs";

const { screen, mount, planned } = newScreen();
const focused = () => screen.window.document.activeElement;
const step = (host) => host.querySelector("[data-progress] [data-step]").textContent;
const CANCELLED = refusal("generate.cancelled", "отменено");

test("по ходу генерации виден свой шаг и попытка правки N из M, чужие шаги не видны", async () => {
  const { host, emit } = await planned({ answers: { start_program: held } });
  press(host, "[data-start]");
  await settled();

  assert.equal(step(host), ru.generate.working);
  assert.ok(host.querySelector("[data-start]") === null, "карту можно запустить второй раз");
  assert.ok(focused() === host.querySelector("[data-cancel]"), "фокус не на отмене");
  emit(began("sources"));
  assert.equal(step(host), ru.generate.stepSources);
  emit(began("repair", 2, 3));
  assert.equal(step(host), "Исправляю текст: попытка 2 из 3");
  emit({ step: "repair", state: "ended", round: 2, of: 3 });
  emit(began("plan"));
  emit(began("fork"));
  assert.equal(step(host), "Исправляю текст: попытка 2 из 3");
});

test("отмена этапа ждёт ответа генерации: «Останавливаю…», потом тост и фокус на «Начать»", async () => {
  const start = deferred();
  const { host, calls, said, gone } = await planned({ answers: { start_program: start.answer } });
  press(host, "[data-start]");
  await settled();
  press(host, "[data-cancel]");
  await settled();

  assert.equal(step(host), ru.generate.cancelling);
  assert.equal(host.querySelector("[data-cancel]").getAttribute("aria-disabled"), "true");
  press(host, "[data-cancel]");
  await settled();
  assert.equal(named(calls, "cancel_generation").length, 1);
  assert.ok(host.querySelector("[data-start]") === null, "карта вернулась до конца генерации");

  start.reject(CANCELLED);
  await settled();
  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.cancelled });
  assert.ok(focused() === host.querySelector("[data-start]"), "фокус не вернулся на «Начать»");
  assert.ok(host.querySelector("[data-refused]") === null, "отмена показана отказом");
  assert.deepEqual(gone, []);
});

test("опоздавшая отмена говорит «уже нельзя», и этап всё равно открывается", async () => {
  const start = deferred();
  const late = { start_program: start.answer, cancel_generation: { cancelled: false } };
  const { host, said, gone } = await planned({ answers: late });
  press(host, "[data-start]");
  await settled();
  press(host, "[data-cancel]");
  await settled();

  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.late });
  assert.equal(step(host), ru.generate.working);
  assert.ok(host.querySelector("[data-cancel]").getAttribute("aria-disabled") === null);

  start.resolve(STARTED);
  await settled();
  assert.deepEqual(gone, ["/ru/stage/?program=chip&stage=tracker"]);
  assert.ok(!said.some((seen) => seen.text === ru.generate.cancelled), "показано «отменено»");
});

test("отмена карты генерацию не трогает и возвращает фокус на «Составить карту»", async () => {
  const { host, calls, said } = await planned({ answers: { plan_program: held } });
  press(host, "[data-cancel]");
  await settled();

  assert.equal(named(calls, "cancel_generation").length, 0);
  assert.equal(host.querySelector("[data-request]").value, REQUEST);
  assert.ok(host.querySelector("[data-plan-map]") === null);
  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.cancelled });
  assert.ok(focused() === host.querySelector("[data-plan]"), "фокус не на «Составить карту»");
});

test("отмена правки карты оставляет прежнюю карту и фокус на «Изменить запрос»", async () => {
  const { host, calls } = await planned({ answers: { revise_plan: held } });
  fill(host, "[data-wish]", "короче");
  press(host, "[data-revise]");
  await settled();
  press(host, "[data-cancel]");
  await settled();

  assert.equal(named(calls, "cancel_generation").length, 0);
  assert.match(host.querySelector("[data-plan-map] h2").textContent, /Чиптюн с нуля/);
  assert.ok(focused() === host.querySelector("[data-revise]"), "фокус не на «Изменить запрос»");
});

test("отказ провайдера или харнесса называет причину и ведёт в настройки", async () => {
  const cases = [
    [refusal("provider.no-model", "model is not chosen"), ru.provider.noModel],
    [refusal("harness.not-found", "claude: not found"), ru.provider.notFound],
  ];
  for (const [failure, reason] of cases) {
    const { host } = await planned({ answers: { start_program: rejected(failure) } });
    press(host, "[data-start]");
    await settled();

    const refused = host.querySelector("[data-refused]");
    assert.ok(refused.textContent.includes(reason), reason);
    assert.equal(refused.querySelector("[data-to-settings]").getAttribute("href"), "/ru/settings/");
    assert.ok(focused() === refused, "отказ не в фокусе");
    assert.ok(host.querySelector("[data-start]") !== null, "повторить нельзя");
  }
});

test("отказ по содержанию показывает текст генерации и в настройки не шлёт", async () => {
  const unfit = rejected(refusal("generate.unfit", "карта не уложилась в часы"));
  const { host } = await planned({ answers: { start_program: unfit } });
  press(host, "[data-start]");
  await settled();

  assert.match(host.querySelector("[data-refused]").textContent, /карта не уложилась в часы/);
  assert.ok(host.querySelector("[data-to-settings]") === null);
});

test("занятая генерация говорит своим текстом и в настройки не шлёт", async () => {
  const busy = rejected(refusal("generate.busy", "уже идёт генерация"));
  const { host } = await planned({ answers: { start_program: busy } });
  press(host, "[data-start]");
  await settled();

  assert.match(host.querySelector("[data-refused]").textContent, new RegExp(ru.generate.busy));
  assert.ok(host.querySelector("[data-to-settings]") === null);
});

test("ушедший экран отписывается от шагов генерации", () => {
  const { dispose, stops } = mount();
  assert.equal(stops(), 0);
  dispose();
  assert.equal(stops(), 1);
});

test("ожидание карты сразу даёт индикатор, счётчик и текущую отметку в работе", async () => {
  const { host, emit } = await planned({ answers: { plan_program: held } });

  const progress = host.querySelector("[data-progress]");
  assert.ok(progress.querySelector("[data-spin]") !== null, "в ожидании нет живого индикатора");
  assert.equal(progress.querySelector("[data-elapsed]").textContent, "0 с");
  assert.equal(host.querySelector("[data-mark][data-busy]").dataset.mark, "reach");

  emit(began("plan"));
  await settled();

  const busy = host.querySelector("[data-mark][data-busy]");
  assert.equal(busy.dataset.mark, "drawn", "в работе отмечена не текущая отметка");
  assert.equal(busy.querySelector("[data-label]").textContent, ru.generate.stepPlan);
  assert.ok(busy.querySelector("[data-elapsed]") !== null, "у текущей отметки нет счётчика");
  assert.equal(step(host), ru.generate.stepPlan);
});
