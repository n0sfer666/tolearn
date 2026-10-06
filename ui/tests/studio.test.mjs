import assert from "node:assert/strict";
import test from "node:test";

import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { began, deferred, named, press, refusal, rejected } from "./support/generation.mjs";
import { LEVEL, LOG, PLAN, REQUEST, SPLIT, fill, newScreen } from "./support/new.mjs";

const { mount, planned } = newScreen();
const marks = (host) => [...host.querySelectorAll("[data-mark]")].map((row) => row.dataset.mark);
const done = (host) => [...host.querySelectorAll("[data-mark][data-done]")].map((row) => row.dataset.mark);
const timed = (host) => host.querySelectorAll("[data-mark] [data-at]").length;

test("язык программы по умолчанию — язык интерфейса и уходит в карту", async () => {
  const { host, calls } = mount({ text: en, locale: "en" });
  assert.equal(host.querySelector("[data-tongue]").value, "en");

  fill(host, "[data-request]", REQUEST);
  fill(host, "[data-level]", LEVEL);
  press(host, "[data-plan]");
  await settled();

  assert.deepEqual(named(calls, "plan_program")[0].payload, { request: REQUEST, level: LEVEL, locale: "en" });
});

test("выбранный язык уходит и в переделку карты, и в запуск", async () => {
  const { host, calls } = await planned();
  fill(host, "[data-tongue]", "en");
  fill(host, "[data-wish]", "короче");
  press(host, "[data-revise]");
  await settled();
  press(host, "[data-start]");
  await settled();

  assert.equal(named(calls, "plan_program")[0].payload.locale, "ru");
  assert.equal(named(calls, "revise_plan")[0].payload.locale, "en");
  assert.equal(named(calls, "start_program")[0].payload.locale, "en");
});

test("шаги отмечаются по очереди, время идёт только у измеренных", async () => {
  const draw = deferred();
  const { host, emit } = mount({ answers: { plan_program: () => draw.answer() } });
  assert.deepEqual(marks(host), [], "шаги видны до запроса");

  fill(host, "[data-request]", REQUEST);
  fill(host, "[data-level]", LEVEL);
  press(host, "[data-plan]");
  await settled();
  assert.deepEqual(marks(host), ["asked", "reach", "drawn"]);
  assert.deepEqual(done(host), ["asked"], "отмечен не только принятый запрос");
  assert.equal(timed(host), 0, "у принятого запроса нет длительности");

  emit(began("plan"));
  await settled();
  assert.deepEqual(done(host), ["asked", "reach"], "провайдер не отмечен по первому шагу");
  assert.equal(timed(host), 1);

  draw.resolve(PLAN);
  await settled();
  assert.deepEqual(done(host), ["asked", "reach", "drawn"], "карта не отмечена по ответу");
  assert.equal(timed(host), 2, "у карты и провайдера должно быть своё время");
  assert.match(host.querySelector('[data-mark="drawn"] [data-at]').textContent, /\sс$/);
});

test("без шага от провайдера «Сеть и провайдер» отмечается без времени", async () => {
  const { host } = await planned();

  assert.deepEqual(done(host), ["asked", "reach", "drawn"]);
  assert.equal(host.querySelector('[data-mark="reach"] [data-at]'), null, "время взято из воздуха");
  assert.ok(host.querySelector('[data-mark="drawn"] [data-at]') !== null, "карта осталась без времени");
});

test("вкладка журнала считает записи, показывает путь, открывает папку и чистит", async () => {
  const { host, calls } = await planned();
  assert.match(host.querySelector('[data-tab="log"]').textContent, /\(3\)/);
  assert.ok(host.querySelector("[data-journal-room]") === null, "журнал показан поверх карты");
  assert.ok(named(calls, "llm_log").length >= 2, "журнал не перечитан после карты");


  press(host, '[data-tab="log"]');
  await settled();
  assert.match(host.querySelector("[data-journal-room]").textContent, new RegExp(LOG));
  assert.ok(host.querySelector("[data-plan-map]") === null, "карта показана поверх журнала");

  press(host, "[data-journal-open]");
  await settled();
  press(host, "[data-journal-clear]");
  await settled();

  const asks = named(calls, "llm_log").map((made) => made.payload);
  assert.deepEqual(asks.at(-2), { open: true, clear: false });
  assert.deepEqual(asks.at(-1), { open: false, clear: true });
  assert.match(host.querySelector('[data-tab="log"]').textContent, /\(0\)/);
});

test("счётчик журнала перечитывается и после отказа", async () => {
  const failed = rejected(refusal("generate.offline", "нет сети"));
  const { host, calls } = mount({ answers: { plan_program: failed } });
  const before = named(calls, "llm_log").length;

  fill(host, "[data-request]", REQUEST);
  fill(host, "[data-level]", LEVEL);
  press(host, "[data-plan]");
  await settled();

  assert.ok(named(calls, "llm_log").length > before, "отказ тоже пишет в журнал, а счётчик не обновлён");
});

test("карта не прячет форму: запрос, уровень и язык остаются на экране", async () => {
  const { host } = await planned();

  assert.equal(host.querySelector("[data-request]").value, REQUEST);
  assert.equal(host.querySelector("[data-level]").value, LEVEL);
  assert.ok(host.querySelector("[data-tongue]") !== null, "язык программы пропал вместе с формой");
  assert.ok(host.querySelector("[data-plan-map]") !== null);
});

test("правило 70 ч: лист сверх бюджета помечен, а карта из подпрограмм — нет", async () => {
  const { host } = await planned();
  const note = host.querySelector("[data-plan-map] [data-budget]");
  assert.match(note.textContent, /70/);
  assert.equal(note.dataset.over, undefined);

  const leaf = { plan: { ...PLAN.plan, children: [] }, hours: { min: 80, max: 120 } };
  const big = await planned({ answers: { plan_program: leaf } });
  assert.equal(big.host.querySelector("[data-budget]").dataset.over, "true");

  const split = await planned({ answers: { plan_program: SPLIT } });
  assert.equal(
    split.host.querySelector("[data-budget]").dataset.over,
    undefined,
    "карта из подпрограмм дробится, а не выходит за бюджет",
  );
  const rows = [...split.host.querySelectorAll("[data-plan-part]")].map((row) => row.dataset.over);
  assert.deepEqual(rows, [undefined, "true"], "подпрограмма сверх 70 ч не помечена");
});

test("«Начать» строит этап по тому запросу, из которого вышла карта", async () => {
  const { host, calls } = await planned();

  fill(host, "[data-request]", "Хочу писать музыку для кино");
  fill(host, "[data-tongue]", "en");
  press(host, "[data-start]");
  await settled();

  assert.deepEqual(named(calls, "start_program")[0].payload, {
    request: REQUEST,
    level: LEVEL,
    locale: "ru",
    plan: PLAN.plan,
  });
});

test("«Начать» называет первый этап, а у карты из подпрограмм — просто «Начать»", async () => {
  const { host } = await planned();
  assert.equal(host.querySelector("[data-start]").textContent, "Начать: сгенерировать «Трекер и паттерны»");

  const split = await planned({ answers: { plan_program: SPLIT } });
  assert.equal(split.host.querySelector("[data-start]").textContent, ru.generate.start);
});
