import assert from "node:assert/strict";
import test, { after, before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";
import { QUIET, began, deferred, ending, heard, job, named, press, transport } from "./support/generation.mjs";

const HERE = "https://tolearn.local/ru/stage/?program=chip&node=rom&stage=voices";
const MINUTE = 60_000;
let Ongoing;
let render;
let window;
const alive = [];

before(
  async () => {
    window = browser(HERE);
    globalThis.location = window.location;
    ({ default: Ongoing } = await island("Ongoing"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

after(() => {
  for (const dispose of alive) dispose();
});

function mount(options = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const { calls, call } = transport({
    generation_state: options.state ?? QUIET,
    generation_seen: QUIET,
    cancel_generation: { cancelled: true },
    ...options.answers,
  });
  const said = toasts(window);
  const { steps, emit } = heard();
  const watched = heard();
  const watch = options.watch ?? watched.steps;
  const props = { text: ru, locale: "ru", folded: options.folded, call, steps, watch };
  alive.push(render(() => Ongoing(props), host));
  return { host, calls, said, emit, tell: watched.emit };
}

const running = (work) => ({ work, outcome: null });
const text = (host, selector) => host.querySelector(selector)?.textContent ?? null;

test("шапка подписывается на generation-state раньше, чем читает generation_state", async () => {
  const order = [];
  const { calls } = mount({
    watch: () => {
      order.push("watch");
      return () => undefined;
    },
    answers: {
      generation_state: () => {
        order.push("read");
        return Promise.resolve(QUIET);
      },
    },
  });
  await settled();

  assert.deepEqual(order, ["watch", "read"], "подписка случилась после чтения");
  assert.equal(named(calls, "generation_state").length, 1);
});

test("событие, пришедшее во время чтения, побеждает прочитанный ответ", async () => {
  const read = deferred();
  const { host, tell } = mount({ answers: { generation_state: read.answer } });
  await settled();
  tell(running(job("fork")));
  read.resolve(QUIET);
  await settled();

  assert.ok(host.querySelector("[data-ongoing]") !== null, "ход пропал после старого ответа");
});

test("идущая работа в шапке: шаг, время с начала, ссылка на экран запуска и «Отменить» для ⌘K", async () => {
  const work = job("fork", { began: Date.now() - 80_000, marks: [{ step: began("fork"), at: 0 }] });
  const { host, calls } = mount({ state: running(work) });
  await settled();

  assert.equal(text(host, "[data-ongoing] [data-step]"), ru.generate.stepFork);
  assert.equal(text(host, "[data-ongoing] [data-elapsed]"), "1:20");
  assert.equal(
    host.querySelector("[data-ongoing-link]").getAttribute("href"),
    "/ru/next/?program=chip&node=rom&stage=voices",
  );
  const cancel = host.querySelector("[data-ongoing-cancel]");
  assert.ok(cancel.hasAttribute("data-action"), "отмены нет в ⌘K");
  assert.equal(cancel.getAttribute("aria-label"), ru.generate.cancelGeneration);
  assert.equal(cancel.textContent, ru.generate.cancel);

  press(host, "[data-ongoing-cancel]");
  await settled();
  assert.equal(named(calls, "cancel_generation").length, 1, "шапка не зовёт отмену чужой работы");
});

test("отмена из шапки снимает ход и говорит «Генерация отменена»", async () => {
  const { host, said, tell } = mount({ state: running(job("plan")) });
  await settled();
  press(host, "[data-ongoing-cancel]");
  await settled();
  tell(QUIET);
  await settled();

  assert.ok(host.querySelector("[data-ongoing]") === null, "ход остался после отмены");
  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.cancelled });
});

test("опоздавшая отмена на шаге записи говорит тот же текст, что и на экране запуска", async () => {
  const { host, said } = mount({
    state: running(job("start")),
    answers: { cancel_generation: { cancelled: false } },
  });
  await settled();
  press(host, "[data-ongoing-cancel]");
  await settled();

  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.late });
  assert.ok(host.querySelector("[data-ongoing]") !== null, "запись этапа пропала из шапки");
});

test("свёрнутая шапка этапа — «Генерация · N мин» со ссылкой, без шага и индикатора", async () => {
  const work = job("regenerate", { began: Date.now() - 3 * MINUTE - 5_000 });
  const { host } = mount({ folded: true, state: running(work) });
  await settled();

  assert.equal(text(host, "[data-ongoing-link]"), "Генерация · 3 мин");
  assert.ok(host.querySelector("[data-ongoing] [data-step]") === null, "свёрнутая шапка показывает шаг");
  assert.ok(host.querySelector("[data-ongoing] [data-spin]") === null, "свёрнутая шапка крутит индикатор");
});

test("конец работы, которого экран не видел, — надпись со ссылкой на результат", async () => {
  const plan = job("plan");
  const cases = [
    [ending(job("start"), { stage: { program: "chip", node: "", stage: "intro" } }), ru.generate.readyStage, "/ru/stage/?program=chip&stage=intro"],
    [ending(job("fork", { stage: "pulse" }), { fork: { variants: [] } }), ru.generate.readyFork, "/ru/next/?program=chip&node=rom&stage=pulse"],
    [ending(plan, { plan: { plan: null, hours: null } }), ru.generate.readyMap, "/ru/new/"],
    [ending(job("next"), { refusal: { code: "generate.content", message: "нет" } }), ru.generate.readyRefused, "/ru/next/?program=chip&node=rom&stage=voices"],
  ];
  for (const [outcome, said, href] of cases) {
    const { host } = mount({ state: { work: null, outcome } });
    await settled();
    assert.equal(text(host, "[data-ongoing-open]"), said);
    assert.equal(host.querySelector("[data-ongoing-open]").getAttribute("href"), href);
  }
});

test("× гасит надпись через generation_seen, а просмотренный итог надписи не даёт", async () => {
  const outcome = ending(job("plan"), { plan: { plan: null, hours: null } });
  const { host, calls, tell } = mount({ state: { work: null, outcome } });
  await settled();
  press(host, "[data-ongoing-close]");
  await settled();
  assert.equal(named(calls, "generation_seen").length, 1);

  tell({ work: null, outcome: { ...outcome, seen: true } });
  await settled();
  assert.ok(host.querySelector("[data-ongoing-done]") === null, "надпись осталась после ×");
});

test("открытый готовый этап сам зовёт generation_seen и надписи не показывает", async () => {
  const outcome = ending(job("regenerate"), { stage: { program: "chip", node: "rom", stage: "voices" } });
  const { host, calls } = mount({ state: { work: null, outcome } });
  await settled();

  assert.equal(named(calls, "generation_seen").length, 1, "открытый этап не погашен");
  assert.ok(host.querySelector("[data-ongoing-done]") === null, "надпись на странице самого этапа");
});
