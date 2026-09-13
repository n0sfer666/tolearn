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
  summary: { passed: 1, total: 2, skipped: 1 },
  active: "2026-09-10",
  unread: null,
};

function mount(shelf, words = ru, locale = "ru") {
  const host = window.document.createElement("ul");
  window.document.body.append(host);
  render(() => Shelf({ shelf, text: words.programs, locale }), host);
  return host.querySelector("[data-program]");
}

test("полоса и сводка считают всё дерево программы", () => {
  const card = mount(NES);
  const bar = card.querySelector("progress");

  assert.equal(Number(bar.value), 1);
  assert.equal(bar.getAttribute("max"), "2");
  assert.equal(bar.getAttribute("aria-label"), ru.programs.progress);
  assert.equal(card.querySelector("[data-summary]").textContent, "пройдено 1 из 2, без зачёта 1");
  assert.equal(mount(NES, en, "en").querySelector("[data-summary]").textContent, "passed 1 of 2, 1 without exam");
});

test("дата последней активности — на языке экрана, а без записей её нет", () => {
  assert.match(mount(NES).querySelector("[data-active]").textContent, /10 сентября 2026/);
  assert.match(mount(NES, en, "en").querySelector("[data-active]").textContent, /September 10, 2026/);
  assert.equal(mount({ ...NES, active: null }).querySelector("[data-active]"), null);
});

test("дерево без сгенерированных этапов не рисует полосу из нуля", () => {
  const card = mount({ ...NES, summary: { passed: 0, total: 0, skipped: 0 }, active: null });

  assert.equal(card.querySelector("progress"), null);
  assert.equal(card.querySelector("[data-summary]"), null);
});

test("битое состояние оставляет карточку и называет причину вместо прогресса", () => {
  const card = mount({ ...NES, summary: null, active: null, unread: "state.yaml: схема tolearn/state/9 не знакома" });
  const reason = card.querySelector("[data-unread]");

  assert.ok(card.querySelector("a"), "карточка потеряла ссылку");
  assert.ok(reason.textContent.includes(ru.programs.unread), reason.textContent);
  assert.ok(reason.textContent.includes("схема tolearn/state/9"), reason.textContent);
  assert.equal(card.querySelector("progress"), null);
  assert.equal(card.querySelector("[data-summary]"), null);
});
