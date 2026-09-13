import assert from "node:assert/strict";
import test from "node:test";

import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { named, press } from "./support/generation.mjs";
import { LEVEL, PLAN, REQUEST, REVISED, fill, newScreen } from "./support/new.mjs";

const { screen, mount, planned } = newScreen();
const focused = () => screen.window.document.activeElement;

test("без запроса или уровня карту не просят и говорят, чего не хватает", async () => {
  const { host, calls } = mount();
  press(host, "[data-plan]");
  fill(host, "[data-request]", REQUEST);
  press(host, "[data-plan]");
  await settled();

  assert.equal(named(calls, "plan_program").length, 0);
  assert.match(host.querySelector("[data-refused]").textContent, new RegExp(ru.generate.missing));
  assert.ok(host.querySelector("[data-to-settings]") === null);

  fill(host, "[data-level]", LEVEL);
  press(host, "[data-plan]");
  await settled();
  assert.deepEqual(named(calls, "plan_program")[0].payload, { request: REQUEST, level: LEVEL });
});

test("тот же отказ второй раз заново встаёт в фокус", async () => {
  const { host } = mount();
  press(host, "[data-plan]");
  await settled();
  const first = host.querySelector("[data-refused]");
  assert.ok(focused() === first, "первый отказ не в фокусе");

  host.querySelector("[data-request]").focus();
  press(host, "[data-plan]");
  await settled();
  const second = host.querySelector("[data-refused]");
  assert.ok(second !== first, "отказ не перерисован");
  assert.ok(focused() === second, "повторный отказ не в фокусе");
});

test("карта показывает этапы, подпрограммы и часы и ждёт «Начать»", async () => {
  const { host, calls } = await planned();

  const map = host.querySelector("[data-plan-map]");
  for (const seen of [PLAN.plan.title, PLAN.plan.goal, "Сведение", "Свести трек", "9–13 ч", "3–4 ч"]) {
    assert.ok(map.textContent.includes(seen), seen);
  }
  const rows = [...map.querySelectorAll("[data-plan-stage]")].map((row) => row.dataset.planStage);
  assert.deepEqual(rows, ["tracker", "voices"]);
  assert.ok(focused() === map.querySelector("h2"), "фокус не на карте");
  assert.equal(named(calls, "start_program").length, 0);
});

test("«Изменить запрос» переделывает карту по пожеланию, и так по кругу", async () => {
  const { host, calls } = await planned();
  press(host, "[data-revise]");
  await settled();
  assert.equal(named(calls, "revise_plan").length, 0);
  assert.match(host.querySelector("[data-refused]").textContent, new RegExp(ru.generate.wishMissing));

  fill(host, "[data-wish]", "без трекера");
  press(host, "[data-revise]");
  await settled();
  fill(host, "[data-wish]", "короче");
  press(host, "[data-revise]");
  await settled();

  const [first, second] = named(calls, "revise_plan").map((made) => made.payload);
  assert.deepEqual(first, { request: REQUEST, level: LEVEL, plan: PLAN.plan, wish: "без трекера" });
  assert.deepEqual(second.plan, REVISED.plan);
  assert.equal(second.wish, "короче");
  assert.match(host.querySelector("[data-plan-map] h2").textContent, /без трекера/);
  assert.equal(host.querySelector("[data-wish]").value, "");
});

test("«Начать» строит программу по показанной карте и открывает её первый этап", async () => {
  const { host, calls, gone } = await planned();
  press(host, "[data-start]");
  await settled();

  assert.deepEqual(named(calls, "start_program")[0].payload, { request: REQUEST, level: LEVEL, plan: PLAN.plan });
  assert.deepEqual(gone, ["/ru/stage/?program=chip&stage=tracker"]);

  const bare = await planned({ answers: { start_program: { program: "chip", node: "", stage: "" } } });
  press(bare.host, "[data-start]");
  await settled();
  assert.deepEqual(bare.gone, ["/ru/program/?program=chip"]);
});

test("английский экран говорит по-английски и открывает английский этап", async () => {
  const { host, gone } = await planned({ text: en, locale: "en" });
  assert.equal(host.querySelector("[data-start]").textContent, en.generate.start);
  press(host, "[data-start]");
  await settled();

  assert.deepEqual(gone, ["/en/stage/?program=chip&stage=tracker"]);
});
