import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled, toasts } from "./support/dom.mjs";
import { heard, named, press, refusal } from "./support/generation.mjs";
import { selects } from "./support/pick.mjs";
import { OUT, stageScreen } from "./support/stage.mjs";

const { screen, mount } = stageScreen();

const AT = { program: "chip", node: "chip", stage: "voices" };

const chain = (turns, clear = false) => ({
  chain: 0,
  block: "p1",
  excerpt: "Чип держит",
  fragment: null,
  turns,
  clear,
});

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
  selects(screen.window).focus(host, "p1");
  await settled();
  press(host, "[data-pick='p1']");
  await settled();
  typed(host, value);
  press(host, "[data-clarify='p1'] [data-ask]");
  for (let round = 0; round < 5; round += 1) await settled();
};

test("врезки уточнений стоят под каждым блоком, кроме заголовка", async () => {
  const { host } = opened({});
  await settled();
  const blocks = [...host.querySelectorAll("[data-clarify]")].map((node) => node.getAttribute("data-clarify"));
  assert.deepEqual(blocks, ["p1", "d1", "i1", "k1", "n1"]);
});

test("вопрос уходит одним вызовом, ответ встаёт врезкой под блоком", async () => {
  const { host, calls } = opened({ clarify: answered });
  await settled();
  await asked(host, "Почему пять?");
  assert.deepEqual(named(calls, "clarify")[0].payload, { ...AT, block: "p1", question: "Почему пять?", chain: null, fragment: null });
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

test("врезка к исчезнувшему блоку — в свёрнутом списке в конце этапа с началом исходного текста", async () => {
  const lost = { chain: 1, block: "gone", excerpt: "Прежний абзац про шум", fragment: "случайный шум", turns: [{ asked: null, answer: "Шум — это случайный сигнал" }], clear: true };
  const out = { ...OUT, clarifications: [chain([{ asked: null, answer: "Иначе" }]), lost] };
  const { host } = opened({}, out);
  await settled();
  const orphans = host.querySelector("[data-orphans]");
  assert.equal(orphans.open, false);
  assert.ok(orphans.closest("[data-stage-body]") === null, "список сирот внутри текста этапа");
  assert.match(orphans.querySelector("summary").textContent, new RegExp(ru.stage.orphans));
  const orphan = orphans.querySelector("[data-orphan='1']");
  assert.match(orphan.querySelector("[data-excerpt]").textContent, /Прежний абзац про шум/);
  assert.equal(orphan.querySelector("[data-fragment]").textContent, "случайный шум");
  assert.match(orphan.querySelector("aside[data-clarified]").textContent, /случайный сигнал/);
  assert.equal(orphans.querySelectorAll("[data-orphan]").length, 1);
  assert.equal(host.querySelectorAll("[data-clarify='p1'] [data-chain]").length, 1);
  assert.ok(orphan.querySelector("[data-yes], [data-no]") === null, "у сироты есть продолжение цепочки");
});

test("«Убрать» у сироты удаляет её, пустой список исчезает", async () => {
  const lost = { chain: 0, block: "gone", excerpt: "Прежний абзац", fragment: null, turns: [{ asked: null, answer: "Иначе" }], clear: false };
  const { host, calls } = opened({ unclarify: () => ({ clarifications: [] }) }, { ...OUT, clarifications: [lost] });
  await settled();
  press(host, "[data-orphans] [data-orphan='0'] [data-unclarify]");
  for (let round = 0; round < 3; round += 1) await settled();
  assert.deepEqual(named(calls, "unclarify")[0].payload, { ...AT, chain: 0 });
  assert.ok(host.querySelector("[data-orphans]") === null, "пустой список сирот остался");
});

test("без сирот списка нет", async () => {
  const { host } = opened({}, { ...OUT, clarifications: [chain([{ asked: null, answer: "Иначе" }])] });
  await settled();
  assert.ok(host.querySelector("[data-orphans]") === null, "список сирот без сирот");
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

test("длинный фрагмент в заголовке врезки укорачивается", async () => {
  const long = "шум ".repeat(40).trim();
  const wordy = { ...chain([{ asked: null, answer: "Иначе" }]), fragment: long };
  const { host } = opened({}, { ...OUT, clarifications: [wordy] });
  await settled();
  const summary = host.querySelector("[data-clarify='p1'] [data-chain] summary").textContent;
  assert.ok(summary.length < long.length, summary);
  assert.ok(summary.endsWith("…»"), summary);
  assert.ok(long.startsWith(summary.slice(1, -2)), summary);
});
