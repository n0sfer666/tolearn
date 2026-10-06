import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { en } from "../src/i18n/en.ts";
import { ru } from "../src/i18n/ru.ts";
import { browser } from "./support/dom.mjs";

let Shelf;
let render;
let window;

before(async () => {
  window = browser();
  ({ default: Shelf } = await island("Shelf", "src/components/reading"));
  ({ render } = await import("solid-js/web"));
}, { timeout: 300_000 });

const NES = {
  uuid: "nes-dev",
  title: "Разработка игр для NES",
  goal: "Собрать игру",
  hours: { min: 14, max: 22 },
  stages: 0,
  subprograms: 2,
  summary: { passed: 1, total: 2, skipped: 1 },
  active: "2026-09-10",
  unread: null,
};

const TODAY = new Date(2026, 8, 11, 15, 30);

function mount(shelf, words = ru, locale = "ru") {
  const host = window.document.createElement("ul");
  window.document.body.append(host);
  render(() => Shelf({ shelf, text: words.programs, locale, today: TODAY }), host);
  return host.querySelector("[data-program]");
}

const said = (card, selector) => card.querySelector(selector).textContent;

test("полоса и сводка считают всё дерево программы и стоят отдельно от названия", () => {
  const card = mount(NES);
  const bar = card.querySelector("[data-passed] progress");

  assert.equal(Number(bar.value), 1);
  assert.equal(bar.getAttribute("max"), "2");
  assert.equal(bar.getAttribute("aria-label"), ru.programs.progress);
  assert.equal(said(card, "[data-passed] [data-summary]"), "пройдено 1 из 2, без зачёта 1");
  assert.equal(said(mount(NES, en, "en"), "[data-summary]"), "passed 1 of 2, 1 without exam");
});

test("объём называет подпрограммы контейнера и этапы листа по карте, с часами и датой в одной строке", () => {
  const card = mount(NES);
  const meta = [...card.querySelector("[data-meta]").children].map((part) => part.textContent);

  assert.deepEqual(meta, ["2 подпрограммы", "14–22 ч", "вчера"]);
  assert.equal(said(mount({ ...NES, stages: 3, subprograms: 0 }), "[data-size]"), "3 этапа");
  assert.equal(said(mount({ ...NES, stages: 18, subprograms: 0 }), "[data-size]"), "18 этапов");
  assert.equal(said(mount({ ...NES, stages: 1, subprograms: 0 }, en, "en"), "[data-size]"), "1 stage");
  assert.equal(said(mount(NES, en, "en"), "[data-size]"), "2 subprograms");
});

test("дата последней активности — относительно сегодня на языке экрана, полная — в подсказке", () => {
  const time = mount(NES).querySelector("time[data-active]");

  assert.equal(time.textContent, "вчера");
  assert.equal(time.getAttribute("datetime"), "2026-09-10");
  assert.match(time.getAttribute("title"), /10 сентября 2026/);
  assert.equal(said(mount(NES, en, "en"), "[data-active]"), "yesterday");
  assert.equal(said(mount({ ...NES, active: "2026-09-11" }), "[data-active]"), "сегодня");
  assert.equal(said(mount({ ...NES, active: "2026-09-08" }), "[data-active]"), "3 дня назад");
  assert.equal(said(mount({ ...NES, active: "2026-09-04" }), "[data-active]"), "на прошлой неделе");
  assert.equal(said(mount({ ...NES, active: "2026-08-28" }), "[data-active]"), "2 недели назад");
  assert.equal(said(mount({ ...NES, active: "2026-07-01" }), "[data-active]"), "2 месяца назад");
  assert.equal(said(mount({ ...NES, active: "2025-06-01" }), "[data-active]"), "в прошлом году");
  assert.ok(mount({ ...NES, active: null }).querySelector("[data-active]") === null);
});

test("дерево без сгенерированных этапов не рисует полосу из нуля", () => {
  const card = mount({ ...NES, summary: { passed: 0, total: 0, skipped: 0 }, active: null });

  assert.equal(card.querySelector("progress"), null);
  assert.equal(card.querySelector("[data-summary]"), null);
});

test("битое состояние оставляет карточку и называет причину вместо прогресса", () => {
  const card = mount({ ...NES, summary: null, active: null, unread: "state.yaml: схема tolearn/state/9 не знакома" });
  const reason = card.querySelector("[data-passed] [data-unread]");

  assert.ok(card.querySelector("a"), "карточка потеряла ссылку");
  assert.ok(reason.textContent.includes(ru.programs.unread), reason.textContent);
  assert.ok(reason.textContent.includes("схема tolearn/state/9"), reason.textContent);
  assert.equal(card.querySelector("progress"), null);
  assert.equal(card.querySelector("[data-summary]"), null);
});
