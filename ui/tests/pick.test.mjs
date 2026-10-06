import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { heard, named, press } from "./support/generation.mjs";
import { selects } from "./support/pick.mjs";
import { OUT, stageScreen } from "./support/stage.mjs";

const { screen, mount } = stageScreen();

const AT = { program: "chip", node: "chip", stage: "voices" };

const answered = (payload) => ({
  clarifications: [
    {
      chain: 0,
      block: "p1",
      excerpt: "Чип держит",
      fragment: payload.fragment,
      turns: [{ asked: payload.question || null, answer: "Иначе: пять каналов" }],
      clear: false,
    },
  ],
});

const opened = async () => {
  const shown = mount({ out: OUT, replies: { clarify: answered }, props: { steps: heard().steps } });
  await settled();
  return { ...shown, ...selects(screen.window) };
};

const asked = async (host, block, value) => {
  const at = `[data-clarify='${block}']`;
  const field = host.querySelector(`${at} textarea[data-doubt]`);
  field.value = value;
  field.dispatchEvent(new screen.window.Event("input", { bubbles: true }));
  press(host, `${at} [data-ask]`);
  for (let round = 0; round < 5; round += 1) await settled();
};

test("пока человек читает, над текстом ничего не висит", async () => {
  const { host } = await opened();
  assert.equal(host.querySelector("[data-pick]"), null);
});

test("выделение внутри блока поднимает «Уточнить», фрагмент уходит в вызов и стоит в заголовке врезки", async () => {
  const { host, calls, select } = await opened();
  select(host, "p1", "пять каналов");
  await settled();
  const pick = host.querySelector("[data-pick]");
  assert.equal(pick.getAttribute("data-pick"), "p1");
  assert.equal(pick.textContent, ru.stage.clarify);
  pick.click();
  await settled();
  assert.equal(host.querySelector("[data-clarify='p1'] [data-fragment]").textContent, "пять каналов");
  await asked(host, "p1", "Почему пять?");
  assert.deepEqual(named(calls, "clarify")[0].payload, {
    ...AT,
    block: "p1",
    question: "Почему пять?",
    chain: null,
    fragment: "пять каналов",
  });
  assert.equal(host.querySelector("[data-clarify='p1'] [data-chain] summary").textContent, "«пять каналов»");
});

test("блок в фокусе даёт «Уточнить блок» без фрагмента", async () => {
  const { host, calls, focus } = await opened();
  focus(host, "n1");
  await settled();
  const pick = host.querySelector("[data-pick]");
  assert.equal(pick.getAttribute("data-pick"), "n1");
  assert.equal(pick.textContent, ru.stage.clarifyBlock);
  assert.ok(pick.hasAttribute("data-action"), "кнопка не видна из ⌘K");
  pick.click();
  await settled();
  assert.equal(host.querySelector("[data-clarify='n1'] [data-fragment]"), null);
  await asked(host, "n1", "");
  assert.equal(named(calls, "clarify")[0].payload.fragment, null);
});

test("блоки — остановки табуляции, заголовок — нет", async () => {
  const { host } = await opened();
  const stops = [...host.querySelectorAll("[data-stage-body] [data-block][tabindex='0']")];
  assert.deepEqual(
    stops.map((node) => node.id),
    ["p1", "d1", "i1", "k1", "n1"],
  );
  assert.equal(host.querySelector("#h1").getAttribute("tabindex"), null);
});

test("снятое выделение убирает кнопку", async () => {
  const { host, select, unselect } = await opened();
  select(host, "p1", "пять каналов");
  await settled();
  assert.ok(host.querySelector("[data-pick]") !== null);
  unselect();
  await settled();
  assert.equal(host.querySelector("[data-pick]"), null);
});

test("выделение в заголовке ничего не поднимает", async () => {
  const { host, select } = await opened();
  select(host, "h1", "Пять");
  await settled();
  assert.equal(host.querySelector("[data-pick]"), null);
});

test("выделение через границу двух блоков ничего не поднимает", async () => {
  const { host, spread } = await opened();
  spread(host, "p1", "пять каналов", "n1", "Громкость");
  await settled();
  assert.equal(host.querySelector("[data-pick]"), null);
});
