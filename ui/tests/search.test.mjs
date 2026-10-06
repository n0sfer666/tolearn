import assert from "node:assert/strict";
import test, { before } from "node:test";

import { island } from "../scripts/island.mjs";
import { ru } from "../src/i18n/ru.ts";
import { browser, settled, toasts } from "./support/dom.mjs";
import { named } from "./support/calls.mjs";

let Search;
let render;
let document;

before(
  async () => {
    ({ document } = browser("https://tolearn.local/ru/search/"));
    ({ default: Search } = await island("Search"));
    ({ render } = await import("solid-js/web"));
  },
  { timeout: 300_000 },
);

const CHIPTUNE = "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84";
const NES_DEV = "7a1d4e90-2c3b-4f58-8d6e-1b9a0c5e7f23";
const ROM = "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";

const STAGE = {
  kind: "stage",
  program: CHIPTUNE,
  node: CHIPTUNE,
  node_title: "Chiptune: музыка звукового чипа NES",
  stage: "voices",
  title: "Голоса чипа",
  block: "",
  snippet: "",
};

const BLOCK = {
  kind: "block",
  program: NES_DEV,
  node: ROM,
  node_title: "Первый ROM в cc65",
  stage: "linker",
  title: "Конфиг компоновщика",
  block: "e3fd4291",
  snippet: "Файл nes.cfg говорит ld65, куда в ROM класть каждый сегмент.",
};

const SHELVES = [
  { uuid: CHIPTUNE, title: "Chiptune: музыка звукового чипа NES" },
  { uuid: NES_DEV, title: "Разработка под NES" },
];

function mount(options = {}) {
  const host = document.createElement("div");
  document.body.append(host);
  const calls = [];
  const call = (name, payload) => {
    calls.push({ name, payload });
    if (name === "library") {
      if (options.unlisted === true) return Promise.reject({ code: "library.unreadable", message: "нет" });
      return Promise.resolve({ programs: options.shelves ?? SHELVES, refused: [] });
    }
    if (options.refuse === true) {
      return Promise.reject({ code: "search.unreadable", message: "нет" });
    }
    return Promise.resolve({ hits: options.hits ?? [STAGE, BLOCK], indexed: 0 });
  };
  const said = toasts(document.defaultView);
  render(() => Search({ text: ru, locale: "ru", call }), host);
  return { host, calls, said };
}

function ask(host, text) {
  const input = host.querySelector("[data-query]");
  input.value = text;
  input.dispatchEvent(new host.ownerDocument.defaultView.Event("input", { bubbles: true }));
  host.querySelector("[data-find]").click();
}

function hits(host) {
  return [...host.querySelectorAll("[data-hit]")];
}

test("экран ничего не ищет до запроса", async () => {
  const { host, calls } = mount();
  await settled();

  assert.deepEqual(calls, [], "экран сходил в ядро до запроса");
  assert.equal(hits(host).length, 0);
  assert.equal(host.querySelector("[data-found]"), null, "счётчик нарисован до запроса");
});

test("запрос уходит в ядро без программы: ищется вся библиотека", async () => {
  const { host, calls } = mount();
  ask(host, "голоса");
  await settled();

  assert.deepEqual(named(calls, "search")[0], { name: "search", payload: { query: "голоса", limit: 20 } });
});

test("находка этапа ведёт на экран этапа корня без узла и якоря", async () => {
  const { host } = mount();
  ask(host, "голоса");
  await settled();

  const [stage] = hits(host);
  assert.equal(stage.getAttribute("data-hit"), "stage");
  assert.equal(
    stage.querySelector("a").getAttribute("href"),
    `/ru/stage/?program=${CHIPTUNE}&stage=voices`,
  );
  assert.equal(stage.querySelector("a").textContent, "Голоса чипа");
  assert.equal(stage.querySelector("[data-kind]").textContent, ru.search.stage);
  assert.equal(stage.querySelector("[data-program]").textContent, "Chiptune: музыка звукового чипа NES");
  assert.equal(stage.querySelector("[data-node]") === null, true, "корень назван дважды");
  assert.equal(stage.querySelector("[data-snippet]") === null, true, "пустой фрагмент нарисован");
});

test("находка фрагмента ведёт на этап дочернего узла к блоку по якорю", async () => {
  const { host } = mount();
  ask(host, "ROM");
  await settled();

  const block = hits(host)[1];
  assert.equal(block.getAttribute("data-hit"), "block");
  assert.equal(
    block.querySelector("a").getAttribute("href"),
    `/ru/stage/?program=${NES_DEV}&node=${ROM}&stage=linker#e3fd4291`,
  );
  assert.equal(block.querySelector("[data-kind]").textContent, ru.search.block);
  assert.equal(block.querySelector("[data-program]").textContent, "Разработка под NES");
  assert.equal(block.querySelector("[data-node]").textContent, "Первый ROM в cc65");
  assert.equal(block.querySelector("[data-snippet]").textContent, BLOCK.snippet);
});

test("пустая выдача говорит об этом", async () => {
  const { host } = mount({ hits: [] });
  ask(host, "тарабарщина");
  await settled();

  assert.equal(host.querySelector("[data-nothing]").textContent, ru.search.nothing);
});

test("отказ ядра показан, а не проглочен", async () => {
  const { host, said } = mount({ refuse: true });
  ask(host, "голоса");
  await settled();

  assert.deepEqual(said.at(-1), { tone: "error", text: ru.search.failed });
  assert.equal(hits(host).length, 0);
  assert.equal(host.querySelector("[data-nothing]") === null, true, "отказ выдан за пустую выдачу");
});

test("счёт находок стоит над списком", async () => {
  const { host } = mount();
  ask(host, "голоса");
  await settled();

  const found = host.querySelector("[data-found]");
  const list = host.querySelector("[data-hits]");
  assert.equal(found.textContent, ru.search.found.replace("{n}", "2"));
  assert.equal(
    (found.compareDocumentPosition(list) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0,
    true,
    "счёт оказался под списком",
  );
  assert.equal(host.querySelector("[data-tally]").getAttribute("aria-live"), "polite");
});

test("полная выдача названа усечением, а не всем найденным", async () => {
  const { host } = mount({ hits: Array.from({ length: 20 }, () => STAGE) });
  ask(host, "чип");
  await settled();

  assert.equal(host.querySelector("[data-found]").textContent, ru.search.shown.replace("{n}", "20"));
});

test("библиотека читается на первом запросе, а не при открытии экрана", async () => {
  const { host, calls } = mount();
  await settled();

  assert.deepEqual(calls, []);

  ask(host, "голоса");
  await settled();

  assert.equal(named(calls, "library").length, 1);
});

test("библиотека читается один раз, а не на каждый запрос", async () => {
  const { host, calls } = mount();
  ask(host, "голоса");
  await settled();
  ask(host, "ROM");
  await settled();

  assert.equal(named(calls, "library").length, 1);
  assert.equal(named(calls, "search").length, 2);
});

test("незнакомая программа оставляет находку без подписи корня", async () => {
  const { host } = mount({ shelves: [] });
  ask(host, "голоса");
  await settled();

  const [stage] = hits(host);
  assert.equal(stage.querySelector("[data-program]"), null, "имя корня взято из воздуха");
  assert.equal(stage.querySelector("[data-node]").textContent, STAGE.node_title);
});

test("отказ библиотеки не ломает поиск и не кричит", async () => {
  const { host, said } = mount({ unlisted: true });
  ask(host, "ROM");
  await settled();

  assert.equal(hits(host).length, 2);
  assert.equal(hits(host)[1].querySelector("[data-node]").textContent, "Первый ROM в cc65");
  assert.deepEqual(said, [], "подпись корня поднята до отказа поиска");
});
