import assert from "node:assert/strict";
import test from "node:test";

import { ru } from "../src/i18n/ru.ts";
import { settled } from "./support/dom.mjs";
import { BROKEN, CHIPTUNE, NES, SHELF, programsScreen } from "./support/programs.mjs";

const { mount } = programsScreen();

function type(host, value) {
  const field = host.querySelector("[data-filter]");
  field.value = value;
  field.dispatchEvent(new Event("input", { bubbles: true }));
}

test("карточка несёт название, цель и часы карты", async () => {
  const { host } = mount();
  await settled();

  const card = host.querySelector(`[data-program="${CHIPTUNE}"]`);
  assert.match(card.textContent, /Chiptune: музыка звукового чипа NES/);
  assert.match(card.querySelector("[data-goal]").textContent, /Написать и проиграть/);
  assert.match(card.querySelector("[data-hours]").textContent, /7\D+11/);
});

test("пустая библиотека показывает только импорт", async () => {
  const { host } = mount({ listing: [] });
  await settled();

  assert.ok(host.querySelector("[data-pick]") !== null, "импорт пропал");
  assert.ok(host.querySelector("[data-filter]") === null, "фильтр над пустой библиотекой");
  assert.ok(host.querySelector("h2, h3, ul") === null, "заголовок или список в пустой библиотеке");
  assert.equal(host.querySelectorAll("p").length, host.querySelectorAll(".intake p").length, "подпись вне зоны импорта");
});

test("зона импорта стоит под списком программ", async () => {
  const { host } = mount({ refused: [BROKEN] });
  await settled();

  const list = host.querySelector("ul");
  const zone = host.querySelector(".intake");
  const FOLLOWING = 4;
  assert.ok(list.querySelector("[data-program]") !== null && list.querySelector("[data-broken]") !== null);
  assert.ok((list.compareDocumentPosition(zone) & FOLLOWING) !== 0, "зона импорта выше списка");
});

test("библиотека из одних битых записей показывает их списком", async () => {
  const { host } = mount({ listing: [], refused: [BROKEN] });
  await settled();

  assert.ok(host.querySelector("ul [data-broken]") !== null, "битая запись без программ пропала");
});

test("битая запись библиотеки видна с причиной, а не пропала", async () => {
  const { host } = mount({ refused: [BROKEN] });
  await settled();

  const broken = host.querySelector("[data-broken]");
  assert.match(broken.textContent, /5f0e/);
  assert.match(broken.textContent, /program\.yaml не читается/);
  assert.match(broken.textContent, new RegExp(ru.programs.broken));
});

test("фильтр оставляет только совпавшие программы", async () => {
  const { host } = mount({ listing: [SHELF, NES] });
  await settled();

  type(host, "nes");
  await settled();
  assert.ok(host.querySelector('[data-program="nes-dev"]'), "совпавшая программа пропала");
  assert.ok(host.querySelector(`[data-program="${CHIPTUNE}"]`), "совпадение по NES в названии потеряно");

  type(host, "игр");
  await settled();
  assert.equal(host.querySelector(`[data-program="${CHIPTUNE}"]`), null);
});

test("фильтр прячет и битые записи, которые не совпали", async () => {
  const { host } = mount({ refused: [BROKEN] });
  await settled();

  type(host, "chiptune");
  await settled();
  assert.equal(host.querySelector("[data-broken]"), null, "битая запись пережила фильтр");

  type(host, "5f0e");
  await settled();
  assert.ok(host.querySelector("[data-broken]"), "совпавшая битая запись пропала");
});

test("ссылка на программу ведёт на её экран текущего языка", async () => {
  const { host } = mount({ locale: "en" });
  await settled();

  assert.equal(host.querySelector("li[data-program] a").getAttribute("href"), `/en/program/?program=${CHIPTUNE}`);
});
