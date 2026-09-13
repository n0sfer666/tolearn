import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled, toasts } from "./support/dom.mjs";
import { heard, named, press, refusal } from "./support/generation.mjs";
import { OUT, stageScreen } from "./support/stage.mjs";

const { screen, mount } = stageScreen();

const AT = { program: "chip", node: "chip", stage: "voices" };

const chain = (turns, clear = false) => ({ chain: 0, block: "p1", excerpt: "Чип держит", turns, clear });

const answered = (payload) => ({
  clarifications: [chain([{ asked: payload.question || null, answer: "Иначе: **пять** каналов" }])],
});

const opened = (replies, out = OUT) => mount({ out, replies, props: { steps: heard().steps } });

const typed = (host, value) => {
  const field = host.querySelector("[data-clarify='p1'] textarea[data-doubt]");
  field.value = value;
  field.dispatchEvent(new screen.window.Event("input", { bubbles: true }));
};

const asked = async (host, value) => {
  press(host, "[data-clarify='p1'] [data-clarify-open]");
  await settled();
  typed(host, value);
  press(host, "[data-clarify='p1'] [data-ask]");
  for (let round = 0; round < 5; round += 1) await settled();
};

test("«Уточнить» стоит под каждым блоком, кроме заголовка", async () => {
  const { host } = opened({});
  await settled();
  const blocks = [...host.querySelectorAll("[data-clarify]")].map((node) => node.getAttribute("data-clarify"));
  assert.deepEqual(blocks, ["p1", "d1", "i1", "k1", "n1"]);
});

test("вопрос уходит одним вызовом, ответ встаёт врезкой под блоком", async () => {
  const { host, calls } = opened({ clarify: answered });
  await settled();
  await asked(host, "Почему пять?");
  assert.deepEqual(named(calls, "clarify")[0].payload, { ...AT, block: "p1", question: "Почему пять?", chain: null });
  const aside = host.querySelector("[data-clarify='p1'] aside[data-clarified]");
  assert.match(aside.textContent, /пять/);
  assert.match(host.querySelector("[data-clarify='p1'] [data-asked]").textContent, /Почему пять\?/);
});

test("«Нет» продолжает цепочку, «Да» закрывает её", async () => {
  const understood = () => ({ clarifications: [chain([{ asked: null, answer: "Иначе" }], true)] });
  const { host, calls } = opened({ clarify: answered, understood });
  await settled();
  await asked(host, "");
  press(host, "[data-clarify='p1'] [data-no]");
  await settled();
  typed(host, "А подробнее?");
  press(host, "[data-clarify='p1'] [data-chain] [data-ask]");
  for (let round = 0; round < 5; round += 1) await settled();
  assert.equal(named(calls, "clarify")[1].payload.chain, 0);
  assert.equal(named(calls, "clarify")[1].payload.question, "А подробнее?");
  press(host, "[data-clarify='p1'] [data-yes]");
  for (let round = 0; round < 3; round += 1) await settled();
  assert.deepEqual(named(calls, "understood")[0].payload, { ...AT, chain: 0 });
  assert.equal(host.querySelector("[data-clarify='p1'] [data-chain]").open, false);
  assert.equal(host.querySelector("[data-clarify='p1'] [data-yes]"), null);
});

test("«Убрать» удаляет цепочку", async () => {
  const out = { ...OUT, clarifications: [chain([{ asked: null, answer: "Иначе" }], true)] };
  const { host, calls } = opened({ unclarify: () => ({ clarifications: [] }) }, out);
  await settled();
  press(host, "[data-clarify='p1'] [data-unclarify]");
  for (let round = 0; round < 3; round += 1) await settled();
  assert.deepEqual(named(calls, "unclarify")[0].payload, { ...AT, chain: 0 });
  assert.equal(host.querySelector("[data-clarify='p1'] [data-chain]"), null);
});

test("отказ показывает причину, сбой «Да» — тост", async () => {
  const out = { ...OUT, clarifications: [chain([{ asked: null, answer: "Иначе" }])] };
  const { host } = opened(
    {
      clarify: () => Promise.reject(refusal("clarification.absent", "нет")),
      understood: () => Promise.reject(new Error("диск")),
    },
    out,
  );
  await settled();
  await asked(host, "");
  assert.ok(host.querySelector("[data-clarify='p1']").textContent.includes(ru.stage.clarifyGone));
  const said = toasts(screen.window);
  press(host, "[data-clarify='p1'] [data-yes]");
  for (let round = 0; round < 3; round += 1) await settled();
  assert.ok(JSON.stringify(said).includes(ru.stage.clarifyFailed));
});
