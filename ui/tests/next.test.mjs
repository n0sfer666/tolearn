import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";
import { began, deferred, heard, named, press, refusal, rejected, sequence, transport } from "./support/generation.mjs";

let Next;
let render;
let window;

before(
  async () => {
    window = browser("https://tolearn.local/ru/next/?program=chip&node=rom&stage=voices");
    globalThis.location = window.location;
    ({ default: Next } = await island("Next"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const span = (min, max) => ({ min, max });
const FORK = {
  variants: [
    { id: "noise", title: "Шумовой канал", hours: span(2, 3), why: "Без шума нет барабанов", recommended: true },
    { id: "dpcm", title: "Сэмплы DPCM", hours: span(3, 5), why: "Живые барабаны", recommended: false },
  ],
};
const CANCELLED = refusal("generate.cancelled", "отменено");
const focused = () => window.document.activeElement;

function mount(options = {}) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const gone = [];
  const { calls, call } = transport({
    fork: FORK,
    take_next: { node: "rom", stage: "dpcm" },
    cancel_generation: { cancelled: true },
    ...options.answers,
  });
  const said = toasts(window);
  const { steps, emit } = heard();
  const props = {
    text: options.text ?? ru,
    locale: options.locale ?? "ru",
    call,
    steps,
    go: (href) => gone.push(href),
    ...options.props,
  };
  const dispose = render(() => Next(props), host);
  return { host, calls, said, gone, dispose, emit };
}

test("развилка берёт этап из адреса и отмечает рекомендованный вариант", async () => {
  const { host, calls } = mount();
  await settled();

  assert.deepEqual(named(calls, "fork")[0].payload, { program: "chip", node: "rom", stage: "voices" });
  assert.equal(host.querySelector("h2").textContent, ru.generate.variants);
  const ids = [...host.querySelectorAll("[data-variant]")].map((row) => row.dataset.variant);
  assert.deepEqual(ids, ["noise", "dpcm"]);
  const noise = host.querySelector('[data-variant="noise"]');
  const dpcm = host.querySelector('[data-variant="dpcm"]');
  assert.ok(noise.hasAttribute("data-recommended"));
  assert.ok(noise.textContent.includes(ru.generate.recommended));
  assert.ok(noise.textContent.includes("Без шума нет барабанов"));
  assert.ok(!dpcm.hasAttribute("data-recommended"));
  assert.ok(!dpcm.textContent.includes(ru.generate.recommended));
  assert.ok(dpcm.textContent.includes("3–5 ч"));
  assert.ok(focused() === host.querySelector("[data-variants]"), "фокус не на вариантах");
});

test("выбор строит выбранный этап и открывает его", async () => {
  const { host, calls, gone } = mount();
  await settled();
  press(host, '[data-variant="dpcm"] [data-choose]');
  await settled();

  assert.deepEqual(named(calls, "take_next")[0].payload, { program: "chip", node: "rom", stage: "voices", choice: 1 });
  assert.deepEqual(gone, ["/ru/stage/?program=chip&node=rom&stage=dpcm"]);
});

test("переход к следующей подпрограмме открывает её первый этап", async () => {
  const { host, gone } = mount({ answers: { take_next: { node: "sound", stage: "pulse" } } });
  await settled();
  press(host, '[data-variant="noise"] [data-choose]');
  await settled();

  assert.deepEqual(gone, ["/ru/stage/?program=chip&node=sound&stage=pulse"]);
});

test("пока открывается развилка — её шаг и «Отменить», после отмены — повтор в фокусе", async () => {
  const fork = deferred();
  const { host, calls, said, emit } = mount({ answers: { fork: sequence(fork.answer, () => Promise.resolve(FORK)) } });
  await settled();
  emit(began("fork"));
  emit(began("text"));

  assert.equal(host.querySelector("[data-progress] [data-step]").textContent, ru.generate.stepFork);
  press(host, "[data-cancel]");
  await settled();
  assert.equal(named(calls, "cancel_generation").length, 1);
  assert.equal(host.querySelector("[data-step]").textContent, ru.generate.cancelling);

  fork.reject(CANCELLED);
  await settled();
  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.cancelled });
  assert.ok(host.querySelector("[data-variant]") === null);
  assert.ok(focused() === host.querySelector("[data-retry]"), "фокус не на повторе");

  press(host, "[data-retry]");
  await settled();
  assert.equal(named(calls, "fork").length, 2);
  assert.equal(host.querySelectorAll("[data-variant]").length, 2);
});

test("опоздавшая отмена развилки сразу отбрасывает её ответ, без «этап записывается»", async () => {
  const fork = deferred();
  const late = { fork: sequence(fork.answer, () => Promise.resolve(FORK)), cancel_generation: { cancelled: false } };
  const { host, said } = mount({ answers: late });
  await settled();
  press(host, "[data-cancel]");
  await settled();

  assert.deepEqual(said.at(-1), { tone: "info", text: ru.generate.cancelled });
  assert.ok(!said.some((seen) => seen.text === ru.generate.late), "развилке сказано «этап записывается»");
  assert.ok(focused() === host.querySelector("[data-retry]"), "фокус не на повторе");

  fork.resolve(FORK);
  await settled();
  assert.ok(host.querySelector("[data-variant]") === null, "ответ после отмены показан");
});

test("отказ развилки называет причину, а в настройки ведёт, только когда дело в провайдере", async () => {
  const provider = mount({ answers: { fork: rejected(refusal("provider.unreachable", "no answer")) } });
  await settled();
  assert.match(provider.host.querySelector("[data-refused]").textContent, new RegExp(ru.provider.unreachable));
  assert.ok(provider.host.querySelector("[data-to-settings]") !== null);
  assert.ok(provider.host.querySelector("[data-retry]") !== null);

  const end = mount({ answers: { fork: rejected(refusal("next.end", "карта кончилась")) } });
  await settled();
  assert.match(end.host.querySelector("[data-refused]").textContent, /карта кончилась/);
  assert.ok(end.host.querySelector("[data-to-settings]") === null);
});

test("генерация выбранного этапа показывает шаги, а отмена возвращает варианты в фокус", async () => {
  const take = deferred();
  const { host, gone, emit } = mount({ answers: { take_next: take.answer } });
  await settled();
  press(host, '[data-variant="noise"] [data-choose]');
  await settled();
  emit(began("text"));

  assert.equal(host.querySelector("[data-step]").textContent, ru.generate.stepText);
  assert.ok(host.querySelector("[data-variant]") === null, "варианты можно выбрать второй раз");
  press(host, "[data-cancel]");
  take.reject(CANCELLED);
  await settled();
  assert.equal(host.querySelectorAll("[data-variant]").length, 2);
  assert.ok(focused() === host.querySelector("[data-variants]"), "фокус не на вариантах");
  assert.deepEqual(gone, []);
});

test("без этапа в адресе развилку не открывают и заголовок развилки не показывают", async () => {
  const { host, calls } = mount({ props: { program: "", stage: "" } });
  await settled();

  assert.match(host.querySelector("[data-empty]").textContent, new RegExp(ru.generate.none));
  assert.ok(!host.textContent.includes(ru.generate.variants), "заголовок развилки над пустым экраном");
  assert.deepEqual(calls, []);
});

test("английская развилка говорит по-английски и открывает английский этап", async () => {
  const { host, gone } = mount({ text: en, locale: "en" });
  await settled();
  assert.equal(host.querySelector("h2").textContent, en.generate.variants);
  assert.ok(host.querySelector('[data-variant="noise"]').textContent.includes(en.generate.recommended));
  press(host, '[data-variant="noise"] [data-choose]');
  await settled();

  assert.deepEqual(gone, ["/en/stage/?program=chip&node=rom&stage=dpcm"]);
});
