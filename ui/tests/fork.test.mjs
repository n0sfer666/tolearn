import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { named, press, rejected, refusal } from "./support/generation.mjs";
import { FORK, NODE, STAGE, forkScreen } from "./support/next.mjs";

const { mount, focus, hover, leave } = forkScreen();

const titles = (host) => [...host.querySelectorAll("[data-ahead] [data-row-title]")].map((row) => row.textContent);
const last = (stages) => ({ ...NODE, stages: stages.slice(0, 2) });

test("студия развилки: варианты слева, итог этапа справа, карта под ними", async () => {
  const { host } = mount();
  await settled();

  const studio = host.querySelector("[data-studio]");
  assert.ok(studio !== null, "экран развилки не студия");
  assert.ok(studio.querySelector("[data-studio-ask] [data-variants]") !== null, "варианты не в главной колонке");
  assert.ok(studio.querySelector("[data-studio-side] [data-outcome]") !== null, "итог этапа не в боковой колонке");
  const map = studio.querySelector("[data-ahead]");
  assert.ok(map !== null && map.parentElement === studio, "карта не под колонками");
});

test("карточка варианта называет объём, место на карте и зачем; рекомендованный стоит первым", async () => {
  const shuffled = { variants: [FORK.variants[1], { ...FORK.variants[0] }] };
  const { host, calls } = mount({ answers: { fork: shuffled } });
  await settled();

  const cards = [...host.querySelectorAll("[data-variant]")];
  assert.deepEqual(
    cards.map((card) => card.dataset.variant),
    ["noise", "dpcm"],
    "рекомендованный вариант не первый",
  );
  const noise = cards[0];
  assert.equal(noise.querySelector("strong").textContent, "Шумовой канал");
  assert.equal(noise.querySelector("[data-hours]").textContent, "2–3 ч");
  assert.equal(noise.querySelector("[data-place]").textContent, "Этап 3 из 3");
  assert.ok(noise.textContent.includes("Без шума нет барабанов"));
  assert.equal(noise.querySelector("[data-choose]").textContent, ru.generate.choose);

  press(host, '[data-variant="noise"] [data-choose]');
  await settled();
  assert.equal(named(calls, "take_next")[0].payload.choice, 1, "выбор ушёл с переставленным номером");
});

test("итог прошедшего этапа читается из состояния, а не из модели", async () => {
  const { host, calls } = mount();
  await settled();

  const outcome = host.querySelector("[data-outcome]");
  assert.equal(outcome.querySelector("[data-verdict]").textContent, "засчитано 1 из 2");
  const missed = [...outcome.querySelectorAll("[data-missed] li")].map((row) => row.textContent);
  assert.deepEqual(missed, ["Что делает канал шума?"], "незачтённые вопросы не перечислены");
  assert.match(outcome.querySelector("[data-clarified]").textContent, /2/);
  assert.equal(outcome.querySelector("[data-ticked]").textContent, "Практика: отмечено 1 из 2");

  assert.deepEqual(named(calls, "node")[0].payload, { program: "chip", node: "rom" });
  assert.deepEqual(named(calls, "stage")[0].payload, { program: "chip", node: "rom", stage: "voices" });
  assert.equal(named(calls, "fork").length, 1, "за итогом ходили к модели");
});

test("пропущенный зачёт назван пропуском, а не нулевым счётом", async () => {
  const skipped = {
    node: { ...NODE, stages: [NODE.stages[0], { ...NODE.stages[1], pass: "skip" }, NODE.stages[2]] },
    stage: { ...STAGE, questions: STAGE.questions.map((ask) => ({ ...ask, result: null, missed: [] })) },
  };
  const { host } = mount({ answers: skipped });
  await settled();

  assert.equal(host.querySelector("[data-verdict]").textContent, ru.generate.outcomeSkipped);
  assert.ok(host.querySelector("[data-missed]") === null, "незачтённых нет, а список показан");
});

test("карта после выбора ставит в слот тот вариант, на котором фокус", async () => {
  const { host } = mount();
  await settled();

  assert.deepEqual(titles(host), ["Прямоугольник", "Голоса чипа", "Шумовой канал"], "в слоте не рекомендованный вариант");
  const slot = host.querySelector("[data-ahead] [data-slot]");
  assert.equal(slot.previousElementSibling.dataset.stage, "voices", "слот встал не после пройденного этапа");
  assert.equal(host.querySelector("[data-picked]").dataset.variant, "noise", "выбранная карточка не помечена");

  focus(host, '[data-variant="dpcm"] [data-choose]');
  await settled();
  assert.equal(host.querySelector("[data-ahead] [data-slot] [data-row-title]").textContent, "Сэмплы DPCM");
  assert.equal(host.querySelector("[data-picked]").dataset.variant, "dpcm");
});

test("уведённая мышь возвращает в слот вариант, на котором стоит фокус", async () => {
  const { host } = mount();
  await settled();

  focus(host, '[data-variant="dpcm"] [data-choose]');
  hover(host, '[data-variant="noise"]');
  await settled();
  assert.equal(host.querySelector("[data-ahead] [data-slot] [data-row-title]").textContent, "Шумовой канал");

  leave(host, "[data-variants]");
  await settled();
  assert.equal(host.querySelector("[data-ahead] [data-slot] [data-row-title]").textContent, "Сэмплы DPCM");
  assert.equal(host.querySelector("[data-picked]").dataset.variant, "dpcm");
});

test("на последнем этапе листа вариант стоит за картой, а не строкой в ней", async () => {
  const { host } = mount({ answers: { node: last(NODE.stages) } });
  await settled();

  assert.equal(host.querySelector("[data-variant] [data-place]").textContent, ru.generate.placeOnward);
  assert.deepEqual(titles(host).slice(0, 2), ["Прямоугольник", "Голоса чипа"], "карта листа изменилась");
  assert.ok(host.querySelector("[data-ahead] [data-slot]") === null, "вариант встал строкой чужого узла");
  const onward = host.querySelector("[data-ahead] [data-onward]");
  assert.equal(onward.querySelector("[data-row-title]").textContent, "Шумовой канал");
  assert.equal(onward.querySelector("[data-place]").textContent, ru.generate.placeOnward);
});

test("непрочитанный этап гасит итог, но не карту, и выбрать вариант не мешает", async () => {
  const gone = rejected(refusal("stage.absent", "этапа нет"));
  const { host, calls, gone: went } = mount({ answers: { stage: gone } });
  await settled();

  assert.equal(host.querySelectorAll("[data-variant]").length, 2);
  assert.ok(host.querySelector("[data-outcome]") === null, "итог собран из воздуха");
  assert.ok(host.querySelector("[data-ahead]") !== null, "карта пропала вместе с чужим отказом");
  assert.equal(host.querySelector("[data-variant] [data-place]").textContent, "Этап 3 из 3");

  press(host, '[data-variant="noise"] [data-choose]');
  await settled();
  assert.equal(named(calls, "take_next").length, 1);
  assert.deepEqual(went, ["/ru/stage/?program=chip&node=rom&stage=dpcm"]);
});

test("непрочитанный узел гасит карту и место, но итог этапа остаётся", async () => {
  const gone = rejected(refusal("node.absent", "узла нет"));
  const { host } = mount({ answers: { node: gone } });
  await settled();

  assert.equal(host.querySelector("[data-verdict]").textContent, "засчитано 1 из 2");
  assert.ok(host.querySelector("[data-ahead]") === null, "карта собрана из воздуха");
  assert.ok(host.querySelector("[data-place]") === null, "место на карте взято из воздуха");
});
