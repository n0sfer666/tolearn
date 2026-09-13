import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled } from "./support/dom.mjs";

let Program;
let render;
let window;

before(
  async () => {
    window = browser("https://tolearn.local/ru/program/?program=nes-dev");
    globalThis.location = window.location;
    ({ default: Program } = await island("Program"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const span = (min, max) => ({ min, max });

const ROOT = {
  program: "nes-dev",
  uuid: "nes-dev",
  title: "Разработка игр для NES",
  goal: "Собрать игру для NES и запустить её в эмуляторе",
  level: "Начинающий: знает Python",
  hours: span(14, 22),
  trail: [],
  stages: [],
  children: [
    {
      id: "tools",
      title: "Инструменты сборки",
      hours: span(8, 12),
      ready: true,
      summary: { passed: 1, total: 2, skipped: 1 },
    },
    { id: "sound", title: "Звук и музыка", hours: span(6, 10), ready: false, summary: null },
    { id: "art", title: "Графика", hours: span(4, 6), ready: true, summary: { passed: 0, total: 0, skipped: 0 } },
  ],
  summary: { passed: 1, total: 2, skipped: 1 },
};

function mount(text, locale) {
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const call = () => Promise.resolve(ROOT);
  render(() => Program({ text, locale, call, program: "nes-dev", node: "" }), host);
  return host;
}

test("у сгенерированной подпрограммы — сводка по её поддереву", async () => {
  const host = mount(ru, "ru");
  const english = mount(en, "en");
  await settled();

  assert.equal(host.querySelector('[data-child="tools"] [data-subtree]').textContent, "пройдено 1 из 2, без зачёта 1");
  assert.equal(
    english.querySelector('[data-child="tools"] [data-subtree]').textContent,
    "passed 1 of 2, 1 without exam",
  );
});

test("у несгенерированной — «ждёт генерации» без чисел", async () => {
  const host = mount(ru, "ru");
  await settled();

  const sound = host.querySelector('[data-child="sound"]');
  assert.equal(sound.querySelector("[data-subtree]"), null);
  assert.equal(sound.querySelector("[data-note]").textContent, "ждёт генерации");
  assert.doesNotMatch(sound.textContent, /пройдено/);
});

test("подпрограмма без сгенерированных этапов обходится без сводки", async () => {
  const host = mount(ru, "ru");
  await settled();

  const art = host.querySelector('[data-child="art"]');
  assert.equal(art.querySelector("[data-subtree]"), null);
  assert.equal(art.querySelector("[data-note]"), null);
  assert.ok(art.querySelector("a"), "у сгенерированной подпрограммы пропала ссылка");
});
