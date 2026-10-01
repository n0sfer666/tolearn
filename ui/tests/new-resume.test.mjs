import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { ending, held, job, named, press, refusal, rejected } from "./support/generation.mjs";
import { LEVEL, PLAN, REQUEST, STARTED, fill, newScreen } from "./support/new.mjs";

const { mount, planned } = newScreen();
const asked = { program: "", node: "", stage: "", request: REQUEST, level: LEVEL, locale: "en" };
const value = (host, selector) => host.querySelector(selector).value;
const hint = (host) => host.querySelector("[data-progress] [data-leave]")?.textContent ?? null;
const back = (state) => mount({ answers: { generation_state: state } });

test("идущая карта: форма с запросом, уровнем и языком, под ней ход и подсказка; карту заново не просят", async () => {
  const plan = job("plan", asked);
  const { host, calls, tell } = back({ work: plan, outcome: null });
  await settled();

  assert.equal(value(host, "[data-request]"), REQUEST);
  assert.equal(value(host, "[data-level]"), LEVEL);
  assert.equal(value(host, "[data-tongue]"), "en");
  assert.equal(hint(host), ru.generate.leaveMap);
  assert.equal(named(calls, "plan_program").length, 0, "экран запустил карту второй раз");

  tell({ work: null, outcome: ending(plan, { plan: PLAN }) });
  await settled();
  await settled();
  assert.ok(host.querySelector("[data-plan-map]") !== null, "готовая карта не встала");
  assert.ok(host.querySelector("[data-start]") !== null, "у карты нет «Начать»");
});

test("карта, готовая без экрана, ждёт на нём с «Начать»", async () => {
  const { host } = back({ work: null, outcome: ending(job("plan", asked), { plan: PLAN, seen: true }) });
  await settled();
  await settled();

  assert.equal(value(host, "[data-request]"), REQUEST);
  assert.match(host.querySelector("[data-plan-map]").textContent, /Трекер и паттерны/);
  assert.ok(host.querySelector("[data-start]") !== null);
});

test("«Составить заново» возвращается с прежней картой, пожеланием и ходом", async () => {
  const revise = job("revise", { ...asked, plan: PLAN.plan, wish: "без трекера" });
  const { host, tell } = back({ work: revise, outcome: null });
  await settled();

  assert.match(host.querySelector("[data-plan-map]").textContent, /Трекер и паттерны/);
  assert.equal(hint(host), ru.generate.leaveMap);

  tell({ work: null, outcome: ending(revise, { refusal: refusal("generate.unfit", "не вышло") }) });
  await settled();
  assert.equal(value(host, "[data-wish]"), "без трекера", "пожелание пропало после отказа");
});

test("идущий запуск программы показывает ход и открывает этап, когда тот готов", async () => {
  const start = job("start", { ...asked, plan: PLAN.plan });
  const { host, gone, tell } = back({ work: start, outcome: null });
  await settled();

  assert.equal(hint(host), ru.generate.leaveStage);
  tell({ work: null, outcome: ending(start, { stage: STARTED }) });
  await settled();
  await settled();
  assert.deepEqual(gone, ["/ru/stage/?program=chip&stage=tracker"]);
});

test("отказ, случившийся без экрана, назван причиной под формой и погашен", async () => {
  const failed = ending(job("plan", asked), { refusal: refusal("generate.unfit", "карта не уложилась в часы") });
  const { host, calls } = back({ work: null, outcome: failed });
  await settled();
  await settled();
  await settled();

  assert.match(host.querySelector("[data-refused]").textContent, /карта не уложилась в часы/);
  assert.equal(named(calls, "generation_seen").length, 1, "отказ остался в шапке");
});

test("запуск при проверке ответа говорит своим текстом", async () => {
  const exam = rejected({ ...refusal("generate.busy", "занято"), held: "exam" });
  const { host } = await planned({ answers: { start_program: exam } });
  press(host, "[data-start]");
  await settled();

  assert.match(host.querySelector("[data-refused]").textContent, new RegExp(ru.generate.busyExam));
});

test("готовый этап из итога экран не восстанавливает: его гасит шапка на странице этапа", async () => {
  const { host, calls } = back({ work: null, outcome: ending(job("start", asked), { stage: STARTED }) });
  await settled();

  assert.ok(host.querySelector("[data-progress]") === null);
  assert.equal(named(calls, "generation_seen").length, 0);
});

test("под шагом подсказано, что страницу можно покинуть и где ждать результат", async () => {
  const drawing = await planned({ answers: { plan_program: held } });
  assert.equal(hint(drawing.host), ru.generate.leaveMap);

  const building = await planned({ answers: { start_program: held } });
  press(building.host, "[data-start]");
  await settled();
  assert.equal(hint(building.host), ru.generate.leaveStage);
});

test("изменённый запрос под готовой картой возвращает «Составить карту»", async () => {
  const { host } = await planned();
  await settled();
  assert.ok(host.querySelector("[data-plan]") === null, "кнопка видна при свежей карте");

  fill(host, "[data-request]", "Хочу писать фолк");
  await settled();
  assert.ok(host.querySelector("[data-plan]") !== null, "устаревшую карту не составить заново");
  fill(host, "[data-request]", REQUEST);
  await settled();
  assert.ok(host.querySelector("[data-plan]") === null);
});

test("отмена вернувшейся карты зовёт cancel_generation и говорит «Генерация отменена»", async () => {
  const plan = job("plan", asked);
  const { host, calls, said, tell } = back({ work: plan, outcome: null });
  await settled();

  press(host, "[data-cancel]");
  await settled();
  assert.equal(named(calls, "cancel_generation").length, 1);
  tell({ work: null, outcome: ending(plan, { refusal: refusal("generate.cancelled", "отменено") }) });
  await settled();
  await settled();
  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.cancelled });
});
